//! Aldrin client library
//!
//! This library implements the client side of the Aldrin specification and can be used to connect
//! to Aldrin brokers. It is fully asynchronous (async/await) and doesn't depend on any specific
//! async runtime.
//!
//! Parts of this library should be considered low-level building blocks for the code that can be
//! auto-generated from Aldrin schemas. As an example, performing a function call on a service
//! requires knowing the function's id and uses a polymorphic type to encode the function's
//! arguments. It is generally recommended to rely on the more ergonomic auto-generated code
//! instead.
//!
//! The first entry point is the [`Client`] and it's [`connect`](Client::connect)
//! method. [`Client`]s are parameterized over an [`AsyncTransport`], which abstracts the low-level
//! details of a transport, like e.g. TCP/IP.
//!
//! After establishing a connection, the resulting [`Client`] must be continuously polled (through
//! [`Client::run`]). One way to achieve this is to "spawn" it with an async runtime of your
//! choice. Alternatively, it can also be polled manually.
//!
//! While the [`Client`] is being polled, all interaction with it happens through a [`Handle`],
//! which can be acquired with [`Client::handle`]. The [`Client`] will automatically shut down (as
//! in, the [`Client::run`] future will complete) when the last [`Handle`] has been dropped.
//!
//! # Examples
//!
//! ```
//! use aldrin_client::Client;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     # let broker = aldrin_test::tokio_based::TestBroker::new();
//!     # let mut handle = broker.clone();
//!     # let (async_transport, t2) = aldrin_channel::unbounded();
//!     # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
//!     // Create an AsyncTransport for connecting to the broker.
//!     // let async_transport = ...
//!
//!     // Connect to the broker:
//!     let client = Client::connect(async_transport).await?;
//!     # tokio::spawn(conn.await??.run());
//!
//!     // Acquire a handle and spawn the client:
//!     let handle = client.handle().clone();
//!     let join = tokio::spawn(client.run());
//!
//!     // The client is now fully connected and can be interacted with through the handle.
//!
//!     // Shut down client:
//!     handle.shutdown();
//!     join.await??;
//!
//!     Ok(())
//! }
//! ```

#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

mod error;
mod events;
mod handle;
mod object;
mod objects;
mod request;
mod serial_map;
mod service;
mod services;

/// Re-exports for the code generator.
///
/// This module and its contents are not part of the public API.
#[doc(hidden)]
pub mod codegen {
    pub use aldrin_proto;
    pub use futures_core;
    pub use uuid;
}

use aldrin_proto::{
    AsyncTransport, AsyncTransportExt, CallFunction, CallFunctionReply, CallFunctionResult,
    Connect, ConnectReply, CreateObject, CreateObjectReply, CreateObjectResult, CreateService,
    CreateServiceReply, CreateServiceResult, DestroyObject, DestroyObjectReply,
    DestroyObjectResult, DestroyService, DestroyServiceReply, DestroyServiceResult, EmitEvent,
    Message, ObjectCreatedEvent, ObjectDestroyedEvent, QueryObject, QueryObjectReply,
    QueryObjectResult, QueryServiceVersion, QueryServiceVersionReply, QueryServiceVersionResult,
    ServiceCreatedEvent, ServiceDestroyedEvent, SubscribeEvent, SubscribeEventReply,
    SubscribeEventResult, SubscribeObjects, SubscribeObjectsReply, SubscribeServices,
    SubscribeServicesReply, UnsubscribeEvent, Value,
};
use events::{EventsId, EventsRequest};
use futures_channel::{mpsc, oneshot};
use futures_util::future::{select, Either};
use futures_util::stream::StreamExt;
use request::{
    CreateObjectRequest, DestroyObjectRequest, EmitEventRequest, QueryObjectRequest,
    QueryObjectRequestReply, Request, SubscribeEventRequest, SubscribeObjectsRequest,
    UnsubscribeEventRequest,
};
use serial_map::SerialMap;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[cfg(feature = "codegen")]
pub use aldrin_codegen_macros::generate;
pub use error::{ConnectError, Error, RunError};
pub use events::{Event, Events};
pub use handle::{CallFunctionFuture, Handle, ObjectServices};
pub use object::{Object, ObjectCookie, ObjectId, ObjectUuid};
pub use objects::{ObjectEvent, Objects};
pub use service::{
    FunctionCall, FunctionCallReply, Service, ServiceCookie, ServiceId, ServiceUuid,
};
pub use services::{ServiceEvent, Services};

type FunctionCallReceiver = mpsc::UnboundedReceiver<(u32, Value, u32)>;
type CreateServiceReplySender =
    oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>;
type Subscriptions = HashMap<u32, HashMap<EventsId, mpsc::UnboundedSender<EventsRequest>>>;

/// Aldrin client used to connect to a broker.
///
/// This is the first entry point to `aldrin-client`. A [`Client`] is used to establish a connection
/// to an Aldrin broker. Afterwards, it should be turned into a [`Future`](std::future::Future) with
/// the [`run`](Client::run) method, which must then be continuously polled and run to completion.
///
/// All interaction with a [`Client`] happens asynchronously through one or more
/// [`Handle`s](Handle), which must be acquired with [`Client::handle`] before calling
/// [`Client::run`].
///
/// # Shutdown
///
/// A [`Client`] will automatically shut down when the last [`Handle`] has been dropped. Keep in
/// mind that several other types (such as e.g. [`Object`]) keep an internal [`Handle`]. Use
/// [`Handle::shutdown`] to shut down the [`Client`] manually.
#[derive(Debug)]
#[must_use = "clients do nothing unless you `.await` or poll `Client::run()`"]
pub struct Client<T>
where
    T: AsyncTransport + Unpin,
{
    t: T,
    recv: mpsc::UnboundedReceiver<Request>,
    handle: Option<Handle>,
    create_object: SerialMap<oneshot::Sender<CreateObjectResult>>,
    destroy_object: SerialMap<oneshot::Sender<DestroyObjectResult>>,
    object_events: SerialMap<SubscribeObjectsRequest>,
    create_service: SerialMap<CreateServiceReplySender>,
    destroy_service: SerialMap<(ServiceCookie, oneshot::Sender<DestroyServiceResult>)>,
    service_events: SerialMap<(mpsc::UnboundedSender<ServiceEvent>, SubscribeMode)>,
    function_calls: SerialMap<oneshot::Sender<CallFunctionResult>>,
    services: HashMap<ServiceCookie, mpsc::UnboundedSender<(u32, Value, u32)>>,
    subscribe_event: SerialMap<(
        EventsId,
        ServiceCookie,
        u32,
        oneshot::Sender<SubscribeEventResult>,
    )>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
    broker_subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
    query_object: SerialMap<QueryObjectData>,
    query_service_version: SerialMap<oneshot::Sender<QueryServiceVersionResult>>,
}

impl<T> Client<T>
where
    T: AsyncTransport + Unpin,
{
    /// Creates a client and connects to an Aldrin broker.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let mut handle = broker.clone();
    /// # let (async_transport, t2) = aldrin_channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// // Create an AsyncTransport for connecting to the broker.
    /// // let async_transport = ...
    ///
    /// // Connect to the broker:
    /// let client = Client::connect(async_transport).await?;
    /// # tokio::spawn(conn.await??.run());
    ///
    /// // Acquire a handle and spawn the client:
    /// let handle = client.handle().clone();
    /// let join = tokio::spawn(client.run());
    ///
    /// // The client is now fully connected and can be interacted with through the handle.
    ///
    /// // Shut down client:
    /// handle.shutdown();
    /// join.await??;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(mut t: T) -> Result<Self, ConnectError<T::Error>> {
        t.send_and_flush(Message::Connect(Connect {
            version: aldrin_proto::VERSION,
        }))
        .await?;

        match t.receive().await? {
            Message::ConnectReply(ConnectReply::Ok) => {}
            Message::ConnectReply(ConnectReply::VersionMismatch(v)) => {
                return Err(ConnectError::VersionMismatch(v))
            }
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg)),
        }

        let (send, recv) = mpsc::unbounded();
        Ok(Client {
            t,
            recv,
            handle: Some(Handle::new(send)),
            create_object: SerialMap::new(),
            destroy_object: SerialMap::new(),
            object_events: SerialMap::new(),
            create_service: SerialMap::new(),
            destroy_service: SerialMap::new(),
            service_events: SerialMap::new(),
            function_calls: SerialMap::new(),
            services: HashMap::new(),
            subscribe_event: SerialMap::new(),
            subscriptions: HashMap::new(),
            broker_subscriptions: HashMap::new(),
            query_object: SerialMap::new(),
            query_service_version: SerialMap::new(),
        })
    }

    /// Returns a handle to the client.
    ///
    /// After creating the [`Client`], [`Handle`s](Handle) are the primary entry point for
    /// interacting with it.
    ///
    /// When the last [`Handle`] is dropped, the [`Client`] will automatically shut down.
    pub fn handle(&self) -> &Handle {
        self.handle.as_ref().unwrap()
    }

    /// Runs the client until it shuts down.
    ///
    /// After creating a [`Client`] it is important to run it before calling any method on a
    /// [`Handle`].
    ///
    /// This is a long running method, that will only complete once the [`Client`] has shut down. It
    /// should ideally be spawned on a dedicated task (not for performance or technical reasons, but
    /// for ergonomics).
    ///
    /// # Shutdown
    ///
    /// A running [`Client`] can be shut down manually with [`Handle::shutdown`]. It will also
    /// automatically shut down when the last [`Handle`] has been dropped. Be aware, that some
    /// types (such as e.g. [`Service`]) hold an internal [`Handle`] and will thus keep the
    /// [`Client`] running. [`Client`s](Client) can also be instructed by the broker to shut down.
    pub async fn run(mut self) -> Result<(), RunError<T::Error>> {
        self.handle.take().unwrap();

        loop {
            match select(self.t.receive(), self.recv.next()).await {
                Either::Left((Ok(Message::Shutdown(())), _)) => {
                    self.t.send_and_flush(Message::Shutdown(())).await?;
                    return Ok(());
                }

                Either::Left((Ok(msg), _)) => self.handle_message(msg).await?,
                Either::Left((Err(e), _)) => return Err(e.into()),

                Either::Right((Some(Request::Shutdown), _)) | Either::Right((None, _)) => {
                    self.t.send_and_flush(Message::Shutdown(())).await?;
                    self.drain_transport().await?;
                    return Ok(());
                }

                Either::Right((Some(req), _)) => self.handle_request(req).await?,
            }
        }
    }

    async fn drain_transport(&mut self) -> Result<(), RunError<T::Error>> {
        loop {
            if let Message::Shutdown(()) = self.t.receive().await? {
                return Ok(());
            }
        }
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), RunError<T::Error>> {
        match msg {
            Message::CreateObjectReply(msg) => self.msg_create_object_reply(msg),
            Message::DestroyObjectReply(msg) => self.msg_destroy_object_reply(msg),
            Message::SubscribeObjectsReply(msg) => self.msg_subscribe_objects_reply(msg),
            Message::ObjectCreatedEvent(msg) => self.msg_object_created_event(msg).await,
            Message::ObjectDestroyedEvent(msg) => self.msg_object_destroyed_event(msg).await,
            Message::CreateServiceReply(msg) => self.msg_create_service_reply(msg),
            Message::DestroyServiceReply(msg) => self.msg_destroy_service_reply(msg),
            Message::SubscribeServicesReply(msg) => self.msg_subscribe_services_reply(msg),
            Message::ServiceCreatedEvent(msg) => self.msg_service_created_event(msg).await,
            Message::ServiceDestroyedEvent(msg) => self.msg_service_destroyed_event(msg).await,
            Message::CallFunction(msg) => self.msg_call_function(msg).await,
            Message::CallFunctionReply(msg) => self.msg_call_function_reply(msg),
            Message::SubscribeEvent(msg) => self.msg_subscribe_event(msg),
            Message::SubscribeEventReply(msg) => self.msg_subscribe_event_reply(msg),
            Message::UnsubscribeEvent(msg) => self.msg_unsubscribe_event(msg),
            Message::EmitEvent(msg) => self.msg_emit_event(msg),
            Message::QueryObjectReply(msg) => self.msg_query_object_reply(msg),
            Message::QueryServiceVersionReply(msg) => self.msg_query_service_version_reply(msg),

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObject(_)
            | Message::DestroyObject(_)
            | Message::SubscribeObjects(_)
            | Message::UnsubscribeObjects(())
            | Message::CreateService(_)
            | Message::DestroyService(_)
            | Message::SubscribeServices(_)
            | Message::UnsubscribeServices(())
            | Message::QueryObject(_)
            | Message::QueryServiceVersion(_) => Err(RunError::UnexpectedMessageReceived(msg)),

            Message::Shutdown(()) => unreachable!(), // Handled in run.
        }
    }

    fn msg_create_object_reply(
        &mut self,
        msg: CreateObjectReply,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(send) = self.create_object.remove(msg.serial) {
            send.send(msg.result).ok();
        }

        Ok(())
    }

    fn msg_destroy_object_reply(
        &mut self,
        msg: DestroyObjectReply,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(send) = self.destroy_object.remove(msg.serial) {
            send.send(msg.result).ok();
        }

        Ok(())
    }

    fn msg_subscribe_objects_reply(
        &mut self,
        msg: SubscribeObjectsReply,
    ) -> Result<(), RunError<T::Error>> {
        let req = match self.object_events.get_mut(msg.serial) {
            Some(req) => req,
            None => return Ok(()),
        };

        if req.mode == SubscribeMode::CurrentOnly {
            self.object_events.remove(msg.serial);
        }

        Ok(())
    }

    async fn msg_object_created_event(
        &mut self,
        msg: ObjectCreatedEvent,
    ) -> Result<(), RunError<T::Error>> {
        let obj_ev = ObjectEvent::Created(ObjectId::new(
            ObjectUuid(msg.uuid),
            ObjectCookie(msg.cookie),
        ));

        if let Some(serial) = msg.serial {
            if let Some(req) = self.object_events.get_mut(serial) {
                if req.sender.unbounded_send(obj_ev).is_err() {
                    self.object_events.remove(serial);
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, req) in self.object_events.iter_mut() {
                if req.sender.unbounded_send(obj_ev).is_err() {
                    remove.push(serial);
                }
            }

            for serial in remove {
                self.object_events.remove(serial);
            }
        }

        if self.object_events.is_empty() {
            self.t.send(Message::UnsubscribeObjects(())).await?;
        }

        Ok(())
    }

    async fn msg_object_destroyed_event(
        &mut self,
        msg: ObjectDestroyedEvent,
    ) -> Result<(), RunError<T::Error>> {
        let obj_ev = ObjectEvent::Destroyed(ObjectId::new(
            ObjectUuid(msg.uuid),
            ObjectCookie(msg.cookie),
        ));

        let mut remove = Vec::new();

        for (serial, req) in self.object_events.iter_mut() {
            if req.sender.unbounded_send(obj_ev).is_err() {
                remove.push(serial);
            }
        }

        for serial in remove {
            self.object_events.remove(serial);
        }

        if self.object_events.is_empty() {
            self.t.send(Message::UnsubscribeObjects(())).await?;
        }

        Ok(())
    }

    fn msg_create_service_reply(
        &mut self,
        msg: CreateServiceReply,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(rep_send) = self.create_service.remove(msg.serial) {
            let recv = if let CreateServiceResult::Ok(cookie) = msg.result {
                let (send, recv) = mpsc::unbounded();
                let dup = self.services.insert(ServiceCookie(cookie), send);
                debug_assert!(dup.is_none());
                Some(recv)
            } else {
                None
            };

            rep_send.send((msg.result, recv)).ok();
        }

        Ok(())
    }

    fn msg_destroy_service_reply(
        &mut self,
        msg: DestroyServiceReply,
    ) -> Result<(), RunError<T::Error>> {
        let (cookie, send) = match self.destroy_service.remove(msg.serial) {
            Some(data) => data,
            None => return Ok(()),
        };

        if let DestroyServiceResult::Ok = msg.result {
            let contained = self.services.remove(&cookie);
            debug_assert!(contained.is_some());
            self.broker_subscriptions.remove(&cookie);
        }

        send.send(msg.result).ok();
        Ok(())
    }

    fn msg_subscribe_services_reply(
        &mut self,
        msg: SubscribeServicesReply,
    ) -> Result<(), RunError<T::Error>> {
        if let Some((_, SubscribeMode::CurrentOnly)) = self.service_events.get_mut(msg.serial) {
            self.service_events.remove(msg.serial);
        }

        Ok(())
    }

    async fn msg_service_created_event(
        &mut self,
        msg: ServiceCreatedEvent,
    ) -> Result<(), RunError<T::Error>> {
        let svc_ev = ServiceEvent::Created(ServiceId::new(
            ObjectId::new(ObjectUuid(msg.object_uuid), ObjectCookie(msg.object_cookie)),
            ServiceUuid(msg.uuid),
            ServiceCookie(msg.cookie),
        ));

        if let Some(serial) = msg.serial {
            if let Some((send, _)) = self.service_events.get_mut(serial) {
                if send.unbounded_send(svc_ev).is_err() {
                    self.service_events.remove(serial);
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, (send, _)) in self.service_events.iter_mut() {
                if send.unbounded_send(svc_ev).is_err() {
                    remove.push(serial);
                }
            }

            for serial in remove {
                self.service_events.remove(serial);
            }
        }

        if self.service_events.is_empty() {
            self.t.send(Message::UnsubscribeServices(())).await?;
        }

        Ok(())
    }

    async fn msg_service_destroyed_event(
        &mut self,
        msg: ServiceDestroyedEvent,
    ) -> Result<(), RunError<T::Error>> {
        let mut remove = Vec::new();
        let service_cookie = ServiceCookie(msg.cookie);

        // A ServiceDestroyedEvent can also be sent, when we have active subscriptions on a
        // service. If that is the sole reason for this event, then make sure not to send
        // UnsubscribeServicesDestroyed below.
        if !self.service_events.is_empty() {
            let svc_ev = ServiceEvent::Destroyed(ServiceId::new(
                ObjectId::new(ObjectUuid(msg.object_uuid), ObjectCookie(msg.object_cookie)),
                ServiceUuid(msg.uuid),
                ServiceCookie(msg.cookie),
            ));

            for (serial, (send, _)) in self.service_events.iter_mut() {
                if send.unbounded_send(svc_ev).is_err() {
                    remove.push(serial);
                }
            }

            for serial in remove {
                self.service_events.remove(serial);
            }

            if self.service_events.is_empty() {
                self.t.send(Message::UnsubscribeServices(())).await?;
            }
        }

        self.broker_subscriptions.remove(&service_cookie);

        if let Some(ids) = self.subscriptions.remove(&service_cookie) {
            let mut dups = HashSet::new();
            for (_, events_ids) in ids {
                for (events_id, sender) in events_ids {
                    if dups.insert(events_id) {
                        // Should we close the channel in case of send errors?
                        sender
                            .unbounded_send(EventsRequest::ServiceDestroyed(service_cookie))
                            .ok();
                    }
                }
            }
        }

        Ok(())
    }

    async fn msg_call_function(&mut self, msg: CallFunction) -> Result<(), RunError<T::Error>> {
        let cookie = ServiceCookie(msg.service_cookie);
        let send = self.services.get_mut(&cookie).expect("inconsistent state");

        if send
            .unbounded_send((msg.function, msg.args, msg.serial))
            .is_err()
        {
            self.t
                .send_and_flush(Message::CallFunctionReply(CallFunctionReply {
                    serial: msg.serial,
                    result: CallFunctionResult::InvalidService,
                }))
                .await?;
        }

        Ok(())
    }

    fn msg_call_function_reply(
        &mut self,
        msg: CallFunctionReply,
    ) -> Result<(), RunError<T::Error>> {
        let send = self
            .function_calls
            .remove(msg.serial)
            .expect("inconsistent state");
        send.send(msg.result).ok();
        Ok(())
    }

    fn msg_subscribe_event(&mut self, msg: SubscribeEvent) -> Result<(), RunError<T::Error>> {
        self.broker_subscriptions
            .entry(ServiceCookie(msg.service_cookie))
            .or_default()
            .insert(msg.event);
        Ok(())
    }

    fn msg_subscribe_event_reply(
        &mut self,
        msg: SubscribeEventReply,
    ) -> Result<(), RunError<T::Error>> {
        let (events_id, service_cookie, id, rep_send) =
            match self.subscribe_event.remove(msg.serial) {
                Some(req) => req,
                None => return Ok(()),
            };

        // || would short-circuit and changing the order would move msg.result out.
        let mut err = msg.result.is_err();
        err |= rep_send.send(msg.result).is_err();
        if err {
            self.subscriptions
                .entry(service_cookie)
                .or_default()
                .entry(id)
                .or_default()
                .remove(&events_id);
        }

        Ok(())
    }

    fn msg_unsubscribe_event(&mut self, msg: UnsubscribeEvent) -> Result<(), RunError<T::Error>> {
        let service_cookie = ServiceCookie(msg.service_cookie);
        let mut subs = match self.broker_subscriptions.entry(service_cookie) {
            Entry::Occupied(subs) => subs,
            Entry::Vacant(_) => return Ok(()),
        };

        subs.get_mut().remove(&msg.event);
        if subs.get().is_empty() {
            subs.remove();
        }

        Ok(())
    }

    fn msg_emit_event(&mut self, msg: EmitEvent) -> Result<(), RunError<T::Error>> {
        let service_cookie = ServiceCookie(msg.service_cookie);
        let senders = match self
            .subscriptions
            .get_mut(&service_cookie)
            .and_then(|s| s.get_mut(&msg.event))
        {
            Some(senders) => senders,
            None => return Ok(()),
        };

        for sender in senders.values_mut() {
            // Should we close the channel in case of send errors?
            sender
                .unbounded_send(EventsRequest::EmitEvent(
                    service_cookie,
                    msg.event,
                    msg.args.clone(),
                ))
                .ok();
        }

        Ok(())
    }

    fn msg_query_object_reply(&mut self, msg: QueryObjectReply) -> Result<(), RunError<T::Error>> {
        let data = match self.query_object.get_mut(msg.serial) {
            Some(data) => data,
            None => return Ok(()),
        };

        if let Some(id_reply) = data.id_reply.take() {
            match msg.result {
                QueryObjectResult::Cookie(cookie) => {
                    if data.with_services {
                        let (send, recv) = mpsc::unbounded();
                        data.svc_reply = Some(send);
                        id_reply.send(Some((ObjectCookie(cookie), Some(recv)))).ok();
                    } else {
                        id_reply.send(Some((ObjectCookie(cookie), None))).ok();
                        self.query_object.remove(msg.serial);
                    }
                    Ok(())
                }

                QueryObjectResult::InvalidObject => {
                    id_reply.send(None).ok();
                    self.query_object.remove(msg.serial);
                    Ok(())
                }

                _ => Err(RunError::UnexpectedMessageReceived(
                    Message::QueryObjectReply(msg),
                )),
            }
        } else if data.with_services {
            match msg.result {
                QueryObjectResult::Service { uuid, cookie } => {
                    let svc_reply = data.svc_reply.as_ref().unwrap();
                    svc_reply
                        .unbounded_send((ServiceUuid(uuid), ServiceCookie(cookie)))
                        .ok();
                    Ok(())
                }

                QueryObjectResult::Done => {
                    self.query_object.remove(msg.serial);
                    Ok(())
                }

                _ => Err(RunError::UnexpectedMessageReceived(
                    Message::QueryObjectReply(msg),
                )),
            }
        } else {
            unreachable!()
        }
    }

    fn msg_query_service_version_reply(
        &mut self,
        msg: QueryServiceVersionReply,
    ) -> Result<(), RunError<T::Error>> {
        let send = match self.query_service_version.remove(msg.serial) {
            Some(send) => send,
            None => return Ok(()),
        };

        send.send(msg.result).ok();
        Ok(())
    }

    async fn handle_request(&mut self, req: Request) -> Result<(), RunError<T::Error>> {
        match req {
            Request::CreateObject(req) => self.req_create_object(req).await,
            Request::DestroyObject(req) => self.req_destroy_object(req).await,
            Request::SubscribeObjects(req) => self.req_subscribe_objects(req).await,
            Request::CreateService(object_cookie, service_uuid, version, reply) => {
                self.req_create_service(object_cookie, service_uuid, version, reply)
                    .await
            }
            Request::DestroyService(cookie, reply) => self.req_destroy_service(cookie, reply).await,
            Request::SubscribeServices(sender, mode) => {
                self.req_subscribe_services(sender, mode).await
            }
            Request::CallFunction(service_cookie, function, args, reply) => {
                self.req_call_function(service_cookie, function, args, reply)
                    .await
            }
            Request::FunctionCallReply(serial, result) => {
                self.req_function_call_reply(serial, result).await
            }
            Request::SubscribeEvent(req) => self.req_subscribe_event(req).await,
            Request::UnsubscribeEvent(req) => self.req_unsubscribe_event(req).await,
            Request::EmitEvent(req) => self.req_emit_event(req).await,
            Request::QueryObject(req) => self.req_query_object(req).await,
            Request::QueryServiceVersion(cookie, reply) => {
                self.req_query_service_version(cookie, reply).await
            }

            // Handled in Client::run()
            Request::Shutdown => unreachable!(),
        }
    }

    async fn req_create_object(
        &mut self,
        req: CreateObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.create_object.insert(req.reply);
        self.t
            .send_and_flush(Message::CreateObject(CreateObject {
                serial,
                uuid: req.uuid.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_object(
        &mut self,
        req: DestroyObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.destroy_object.insert(req.reply);
        self.t
            .send_and_flush(Message::DestroyObject(DestroyObject {
                serial,
                cookie: req.cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_subscribe_objects(
        &mut self,
        req: SubscribeObjectsRequest,
    ) -> Result<(), RunError<T::Error>> {
        let mode = req.mode;
        let is_empty = self.object_events.is_empty();
        let serial = self.object_events.insert(req);
        let (send, serial) = match mode {
            SubscribeMode::All | SubscribeMode::CurrentOnly => (true, Some(serial)),
            SubscribeMode::NewOnly => (is_empty, None),
        };

        if send {
            self.t
                .send_and_flush(Message::SubscribeObjects(SubscribeObjects { serial }))
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn req_create_service(
        &mut self,
        object_cookie: ObjectCookie,
        service_uuid: ServiceUuid,
        version: u32,
        reply: oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.create_service.insert(reply);
        self.t
            .send_and_flush(Message::CreateService(CreateService {
                serial,
                object_cookie: object_cookie.0,
                uuid: service_uuid.0,
                version,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_service(
        &mut self,
        cookie: ServiceCookie,
        reply: oneshot::Sender<DestroyServiceResult>,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.destroy_service.insert((cookie, reply));
        self.t
            .send_and_flush(Message::DestroyService(DestroyService {
                serial,
                cookie: cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_subscribe_services(
        &mut self,
        sender: mpsc::UnboundedSender<ServiceEvent>,
        mode: SubscribeMode,
    ) -> Result<(), RunError<T::Error>> {
        let is_empty = self.service_events.is_empty();
        let serial = self.service_events.insert((sender, mode));
        let (send, serial) = match mode {
            SubscribeMode::All | SubscribeMode::CurrentOnly => (true, Some(serial)),
            SubscribeMode::NewOnly => (is_empty, None),
        };

        if send {
            self.t
                .send_and_flush(Message::SubscribeServices(SubscribeServices { serial }))
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn req_call_function(
        &mut self,
        service_cookie: ServiceCookie,
        function: u32,
        args: Value,
        reply: oneshot::Sender<CallFunctionResult>,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.function_calls.insert(reply);
        self.t
            .send_and_flush(Message::CallFunction(CallFunction {
                serial,
                service_cookie: service_cookie.0,
                function,
                args,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_function_call_reply(
        &mut self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), RunError<T::Error>> {
        self.t
            .send_and_flush(Message::CallFunctionReply(CallFunctionReply {
                serial,
                result,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_subscribe_event(
        &mut self,
        req: SubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        let subs = self
            .subscriptions
            .entry(req.service_cookie)
            .or_default()
            .entry(req.id)
            .or_default();
        let send_req = subs.is_empty();
        let duplicate = subs.insert(req.events_id, req.sender).is_some();

        if send_req {
            let serial = Some(self.subscribe_event.insert((
                req.events_id,
                req.service_cookie,
                req.id,
                req.reply,
            )));

            self.t
                .send_and_flush(Message::SubscribeEvent(SubscribeEvent {
                    serial,
                    service_cookie: req.service_cookie.0,
                    event: req.id,
                }))
                .await
                .map_err(Into::into)
        } else {
            if req.reply.send(SubscribeEventResult::Ok).is_err() && !duplicate {
                subs.remove(&req.events_id);
            }

            Ok(())
        }
    }

    async fn req_unsubscribe_event(
        &mut self,
        req: UnsubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        let mut subs = match self.subscriptions.entry(req.service_cookie) {
            Entry::Occupied(subs) => subs,
            Entry::Vacant(_) => return Ok(()),
        };

        let mut ids = match subs.get_mut().entry(req.id) {
            Entry::Occupied(ids) => ids,
            Entry::Vacant(_) => return Ok(()),
        };

        if ids.get_mut().remove(&req.events_id).is_none() {
            return Ok(());
        }

        if !ids.get().is_empty() {
            return Ok(());
        }

        ids.remove();
        if subs.get().is_empty() {
            subs.remove();
        }

        self.t
            .send_and_flush(Message::UnsubscribeEvent(UnsubscribeEvent {
                service_cookie: req.service_cookie.0,
                event: req.id,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_emit_event(&mut self, req: EmitEventRequest) -> Result<(), RunError<T::Error>> {
        self.t
            .send_and_flush(Message::EmitEvent(EmitEvent {
                service_cookie: req.service_cookie.0,
                event: req.event,
                args: req.args,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_query_object(
        &mut self,
        req: QueryObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.query_object.insert(QueryObjectData {
            object_uuid: req.object_uuid,
            id_reply: Some(req.reply),
            with_services: req.with_services,
            svc_reply: None,
        });
        self.t
            .send_and_flush(Message::QueryObject(QueryObject {
                serial,
                uuid: req.object_uuid.0,
                with_services: req.with_services,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_query_service_version(
        &mut self,
        cookie: ServiceCookie,
        reply: oneshot::Sender<QueryServiceVersionResult>,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.query_service_version.insert(reply);
        self.t
            .send_and_flush(Message::QueryServiceVersion(QueryServiceVersion {
                serial,
                cookie: cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    fn shutdown_all_events(&self) {
        for by_service in self.subscriptions.values() {
            for by_function in by_service.values() {
                for events in by_function.values() {
                    events.close_channel();
                }
            }
        }
    }
}

impl<T> Drop for Client<T>
where
    T: AsyncTransport + Unpin,
{
    fn drop(&mut self) {
        self.shutdown_all_events();
    }
}

/// Mode of subscription for object and service creation events.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubscribeMode {
    /// Receive events for all current and future objects or services.
    All,

    /// Receive events only for current objects or services.
    CurrentOnly,

    /// Receive events only for future objects or services.
    NewOnly,
}

#[derive(Debug)]
struct QueryObjectData {
    object_uuid: ObjectUuid,
    id_reply: Option<oneshot::Sender<QueryObjectRequestReply>>,
    with_services: bool,
    svc_reply: Option<mpsc::UnboundedSender<(ServiceUuid, ServiceCookie)>>,
}
