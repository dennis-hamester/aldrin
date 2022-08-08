pub(crate) mod request;

use crate::channel::{
    PendingReceiver, PendingSender, ReceiverInner, SenderInner, UnclaimedReceiver, UnclaimedSender,
};
use crate::error::InvalidFunctionResult;
use crate::events::{EventsId, EventsRequest};
use crate::{
    Error, Events, Object, ObjectEvent, Objects, Service, ServiceEvent, Services, SubscribeMode,
};
use aldrin_proto::{
    CallFunctionResult, ChannelCookie, ChannelEnd, DestroyObjectResult, FromValue, IntoValue,
    ObjectCookie, ObjectId, ObjectUuid, QueryServiceVersionResult, ServiceCookie, ServiceId,
    ServiceUuid, SubscribeEventResult, Value,
};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_channel::oneshot;
use futures_core::stream::{FusedStream, Stream};
use futures_util::stream::StreamExt;
use request::{
    CallFunctionReplyRequest, CallFunctionRequest, CreateObjectRequest, CreateServiceRequest,
    DestroyObjectRequest, DestroyServiceRequest, EmitEventRequest, HandleRequest,
    QueryObjectRequest, QueryServiceVersionRequest, SubscribeEventRequest, SubscribeObjectsRequest,
    SubscribeServicesRequest, UnsubscribeEventRequest,
};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Handle to a client.
///
/// After connecting a [`Client`](crate::Client) to a broker, [`Handle`s](Handle) are used to
/// interact with it. The first [`Handle`] can be acquired with
/// [`Client::handle`](crate::Client::handle). After that, [`Handle`s](Handle) can be cloned
/// cheaply.
///
/// The [`Client`](crate::Client) will automatically shut down when the last [`Handle`] has been
/// dropped.
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
/// // Connect to the broker:
/// let client = Client::connect(async_transport).await?;
///
/// // Acquire the first handle:
/// let handle = client.handle().clone();
///
/// // Run the client, which consumes it and leaves only the handle for interacting with it:
/// tokio::spawn(client.run());
/// # tokio::spawn(conn.await??.run());
///
/// // Handles are cheap to clone:
/// let handle2 = handle.clone();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Handle {
    send: UnboundedSender<HandleRequest>,
}

impl Handle {
    pub(crate) fn new(send: UnboundedSender<HandleRequest>) -> Self {
        Handle { send }
    }

    /// Shuts down the client.
    ///
    /// Shutdown happens asynchronously, in the sense that when this function returns, the
    /// [`Client`](crate::Client) has only been requested to shut down and not yet necessarily done
    /// so. As soon as [`Client::run`](crate::Client::run) returns, it has fully shut down.
    ///
    /// If the [`Client`](crate::Client) has already shut down (due to any reason), this function
    /// will not treat that as an error. This is different than most other functions, which would
    /// return [`Error::ClientShutdown`] instead.
    pub fn shutdown(&self) {
        self.send.unbounded_send(HandleRequest::Shutdown).ok();
    }

    /// Creates a new object on the bus.
    ///
    /// The `uuid` must not yet exists on the bus, or else [`Error::DuplicateObject`] will be
    /// returned. Use [`ObjectUuid::new_v4`] to create a new random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{Error, ObjectUuid};
    ///
    /// // 6173e119-8066-4776-989b-145a5f16ed4c
    /// const OBJECT2_UUID: ObjectUuid = ObjectUuid::from_u128(0x6173e11980664776989b145a5f16ed4c);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create an object with a random UUID:
    /// let object1 = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Create an object with a fixed UUID:
    /// let object2 = handle.create_object(OBJECT2_UUID).await?;
    ///
    /// // Using the same UUID again will cause an error:
    /// assert_eq!(
    ///     handle.create_object(OBJECT2_UUID).await.unwrap_err(),
    ///     Error::DuplicateObject(OBJECT2_UUID)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_object(&self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateObject(CreateObjectRequest {
                uuid,
                reply: send,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) async fn destroy_object(&self, id: ObjectId) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie: id.cookie,
                reply: send,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        let reply = recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_object_now(&self, cookie: ObjectCookie) {
        let (reply, _) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie,
                reply,
            }))
            .ok();
    }

    /// Subscribes to a stream of object creation and destruction events.
    ///
    /// An [`Objects`] stream can be used to discover and track [`Object`s](Object) on the bus. The
    /// `mode` parameter decides whether the stream will include only current, only new or all
    /// [`Object`s](Object).
    ///
    /// See [`Objects`] for more information and usage examples.
    pub fn objects(&self, mode: SubscribeMode) -> Result<Objects, Error> {
        let (send, recv) = unbounded();
        self.send
            .unbounded_send(HandleRequest::SubscribeObjects(SubscribeObjectsRequest {
                mode,
                sender: send,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(Objects::new(recv))
    }

    pub(crate) async fn create_service(
        &self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
        version: u32,
    ) -> Result<Service, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateService(CreateServiceRequest {
                object_id,
                service_uuid,
                version,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) async fn destroy_service(&self, id: ServiceId) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) fn destroy_service_now(&self, id: ServiceId) {
        let (reply, _) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }))
            .ok();
    }

    /// Subscribes to a stream of service creation and destruction events.
    ///
    /// A [`Services`] stream can be used to discover and track [`Service`s](Service) on the
    /// bus. The `mode` parameter decides whether the stream will include only current, only new or
    /// all [`Service`s](Service).
    ///
    /// See [`Services`] for more information and usage examples.
    pub fn services(&self, mode: SubscribeMode) -> Result<Services, Error> {
        let (sender, recv) = unbounded();
        self.send
            .unbounded_send(HandleRequest::SubscribeServices(SubscribeServicesRequest {
                mode,
                sender,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(Services::new(recv))
    }

    /// Calls a function on a service.
    ///
    /// The function with id `function` will be called with the arguments `args` on the service
    /// identified by `service_id`.
    ///
    /// The returned value of type [`PendingFunctionResult`] is a future which will resolve to the
    /// result of the function call.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_proto::Value;
    /// # use futures::stream::StreamExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_client::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Call function 1 with "1 + 2 = ?" as the argument.
    /// let result = handle.call_function::<_, u32, String>(service_id, 1, "1 + 2 = ?")?;
    /// # svc.next().await.unwrap().reply.ok(3u32)?;
    ///
    /// // Await the result. The `?` here checks for errors on the protocol level, such as a
    /// // intermediate shutdown, or whether the function call was aborted by the callee.
    /// let result = result.await?;
    ///
    /// // Now, result is of type `Result<u32, String>`, directly representing the result of the
    /// // function call.
    /// match result {
    ///     Ok(ok) => assert_eq!(ok, 3),
    ///     Err(err) => panic!("Function call failed: {}.", err),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_function<Args, T, E>(
        &self,
        service_id: ServiceId,
        function: u32,
        args: Args,
    ) -> Result<PendingFunctionResult<T, E>, Error>
    where
        Args: IntoValue,
        T: FromValue,
        E: FromValue,
    {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CallFunction(CallFunctionRequest {
                service_cookie: service_id.cookie,
                function,
                args: args.into_value(),
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(PendingFunctionResult::new(recv, service_id, function))
    }

    /// Calls an infallible function on a service.
    ///
    /// Use this method if the called function is guaranteed to never fail. If this is not true, and
    /// the function fails, then [`Error::InvalidFunctionResult`] will be returned.
    ///
    /// The returned value of type [`PendingFunctionValue`] is a future which will resolve to the
    /// value of the function call.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures::stream::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_client::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Call function 1 with "1 + 2 = ?" as the argument.
    /// let result = handle.call_infallible_function(service_id, 1, "1 + 2 = ?")?;
    /// # svc.next().await.unwrap().reply.ok(3u32)?;
    ///
    /// assert_eq!(3u32, result.await?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_infallible_function<Args, T>(
        &self,
        service_id: ServiceId,
        function: u32,
        args: Args,
    ) -> Result<PendingFunctionValue<T>, Error>
    where
        Args: IntoValue,
        T: FromValue,
    {
        let reply = self.call_function(service_id, function, args)?;
        Ok(PendingFunctionValue(reply))
    }

    pub(crate) fn function_call_reply(
        &self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::CallFunctionReply(CallFunctionReplyRequest {
                serial,
                result,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Creates an Events object used to subscribe to service events.
    ///
    /// See [`Events`] for more information and usage examples.
    pub fn events(&self) -> Events {
        Events::new(self.clone())
    }

    pub(crate) async fn subscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
        sender: UnboundedSender<EventsRequest>,
    ) -> Result<(), Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SubscribeEvent(SubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
                sender,
                reply: rep_send,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = rep_recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            SubscribeEventResult::Ok => Ok(()),
            SubscribeEventResult::InvalidService => Err(Error::InvalidService(service_id)),
        }
    }

    pub(crate) fn unsubscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::UnsubscribeEvent(UnsubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Emits an events to subscribed clients.
    ///
    /// The event with the id `event` of the service identified by `service_id` will be emitted with
    /// the arguments `args` to all subscribed clients.
    ///
    /// Use [`Handle::events`] to subscribe to events.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_client::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Emit event 1 with argument "Hello, world!":
    /// handle.emit_event(service_id, 1, "Hello, world!")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn emit_event(
        &self,
        service_id: ServiceId,
        event: u32,
        args: impl IntoValue,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                args: args.into_value(),
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Waits for an object on the bus.
    ///
    /// Waits for an [`Object`] with the [`ObjectUuid`] `uuid` and returns its [`ObjectId`]. This
    /// function will wait indefinitely until the [`Object`] appears. Use
    /// [`resolve_object`](Handle::resolve_object) search only the currently existing
    /// [`Object`s](Object).
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::ObjectUuid;
    /// use std::time::Duration;
    /// use tokio::time;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object_uuid = ObjectUuid::new_v4();
    ///
    /// // Wait until an object the above UUID appears.
    /// // Awaiting the future now would block indefinitely though.
    /// let object_id = handle.wait_for_object(object_uuid);
    ///
    /// // Create the object:
    /// let object = handle.create_object(object_uuid).await?;
    ///
    /// // Now the future will resolve:
    /// let object_id = object_id.await?;
    /// assert_eq!(object_id, object.id());
    ///
    /// let non_existent = time::timeout(
    ///     Duration::from_millis(500),
    ///     handle.wait_for_object(ObjectUuid::new_v4()),
    /// )
    /// .await;
    /// assert!(non_existent.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_object(&self, uuid: ObjectUuid) -> Result<ObjectId, Error> {
        let mut objects = self.objects(SubscribeMode::All)?;

        while let Some(ev) = objects.next().await {
            match ev {
                ObjectEvent::Created(id) if id.uuid == uuid => return Ok(id),
                _ => {}
            }
        }

        Err(Error::ClientShutdown)
    }

    /// Finds an object with a specific service on the bus.
    ///
    /// Finds an [`Object`], which has a [`Service`] with UUID `service_uuid`. The [`ObjectUuid`]
    /// can be optionally specified as well with `object_uuid`. If it is not specified, then it is
    /// unspecified which [`Object`] the returned [`ServiceId`] belongs to, if any. This function
    /// considers only [`Service`s](Service), which currently exist on the bus. Use
    /// [`wait_for_service`](Handle::wait_for_service) to wait indefinitely for a [`Service`]
    /// matching the criteria.
    ///
    /// If you need more control or even a stream of all [`Object`s](Object) implementing a
    /// particular [`Service`], then use [`services`](Handle::services) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{ObjectUuid, ServiceUuid};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create an object and service:
    /// let object_uuid = ObjectUuid::new_v4();
    /// let object = handle.create_object(object_uuid).await?;
    /// let service_uuid = ServiceUuid::new_v4();
    /// let service = object.create_service(service_uuid, 1).await?;
    ///
    /// // Find a service without specifying an object UUID:
    /// let service_id = handle
    ///     .find_service(service_uuid, None)
    ///     .await?
    ///     .expect("service not found");
    /// // It could be any object (and service cookie), but the service UUID will match:
    /// assert_eq!(service_id.uuid, service.id().uuid);
    ///
    /// // Find a service on a specific object:
    /// let service_id = handle
    ///     .find_service(service_uuid, Some(object_uuid))
    ///     .await?
    ///     .expect("service not found");
    /// // The service id will match:
    /// assert_eq!(service_id, service.id());
    ///
    /// // Searching for a non-existent service yields None:
    /// let non_existent = handle
    ///     .find_service(ServiceUuid::new_v4(), Some(object_uuid))
    ///     .await?;
    /// assert!(non_existent.is_none());
    /// let non_existent = handle
    ///     .find_service(ServiceUuid::new_v4(), None)
    ///     .await?;
    /// assert!(non_existent.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_service(
        &self,
        service_uuid: ServiceUuid,
        object_uuid: Option<ObjectUuid>,
    ) -> Result<Option<ServiceId>, Error> {
        if let Some(object_uuid) = object_uuid {
            self.find_object_service(object_uuid, service_uuid).await
        } else {
            self.find_any_service(service_uuid).await
        }
    }

    async fn find_any_service(
        &self,
        service_uuid: ServiceUuid,
    ) -> Result<Option<ServiceId>, Error> {
        let mut services = self.services(SubscribeMode::CurrentOnly)?;

        while let Some(ev) = services.next().await {
            if let ServiceEvent::Created(id) = ev {
                if id.uuid == service_uuid {
                    return Ok(Some(id));
                }
            }
        }

        Ok(None)
    }

    async fn find_object_service(
        &self,
        object_uuid: ObjectUuid,
        service_uuid: ServiceUuid,
    ) -> Result<Option<ServiceId>, Error> {
        let mut services = match self.query_object_services(object_uuid).await? {
            Some((_, services)) => services,
            None => return Ok(None),
        };

        while let Some(id) = services.next().await {
            if id.uuid == service_uuid {
                return Ok(Some(id));
            }
        }

        Ok(None)
    }

    /// Waits for an object with a specific service on the bus.
    ///
    /// Waits for an [`Object`], which has a [`Service`] with UUID `service_uuid`. The
    /// [`ObjectUuid`] can be optionally specified as well with `object_uuid`. If it is not
    /// specified, then it is unspecified which [`Object`] the returned [`ServiceId`] belongs
    /// to. This function will wait indefinitely until a matching [`Service`] appears. Use
    /// [`find_service`](Handle::find_service) to search only the current [`Service`s](Service) on
    /// the bus.
    ///
    /// If you need more control or even a stream of all [`Object`s](Object) implementing a
    /// particular [`Service`], then use [`services`](Handle::services) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{ObjectUuid, ServiceUuid};
    /// use std::time::Duration;
    /// use tokio::time;
    ///
    /// // 4d090fab-8614-43d1-8473-f29ff84ffc6b
    /// const SERVICE_UUID: ServiceUuid = ServiceUuid::from_u128(0x4d090fab861443d18473f29ff84ffc6b);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Wait until a service with SERVICE_UUID appears.
    /// // Awaiting the future now would block indefinitely though.
    /// let service_id = handle.wait_for_service(SERVICE_UUID, None);
    ///
    /// // Create the object and service:
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    /// let service = object.create_service(SERVICE_UUID, 1).await?;
    ///
    /// // Now the future will resolve:
    /// let service_id = service_id.await?;
    /// // It could be any object (and service cookie), but the service UUID will match:
    /// assert_eq!(service_id.uuid, SERVICE_UUID);
    ///
    /// // Wait for the service on our specific object:
    /// let service_id = handle.wait_for_service(SERVICE_UUID, Some(object.id().uuid)).await?;
    /// assert_eq!(service_id, service.id());
    ///
    /// let non_existent = time::timeout(
    ///     Duration::from_millis(500),
    ///     handle.wait_for_service(ServiceUuid::new_v4(), None),
    /// )
    /// .await;
    /// assert!(non_existent.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_service(
        &self,
        service_uuid: ServiceUuid,
        object_uuid: Option<ObjectUuid>,
    ) -> Result<ServiceId, Error> {
        let mut services = self.services(SubscribeMode::All)?;

        while let Some(ev) = services.next().await {
            let id = match ev {
                ServiceEvent::Created(id) if id.uuid == service_uuid => id,
                _ => continue,
            };

            if let Some(object_uuid) = object_uuid {
                if id.object_id.uuid != object_uuid {
                    continue;
                }
            }

            return Ok(id);
        }

        Err(Error::ClientShutdown)
    }

    /// Resolves an object UUID to its full id.
    ///
    /// This method returns `None`, if the object identified by `object_uuid` doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// use aldrin_client::ObjectUuid;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// let object_id = object.id();
    /// let object_uuid = object_id.uuid;
    ///
    /// let resolved = handle.resolve_object(object_uuid).await?;
    /// assert_eq!(resolved, Some(object_id));
    ///
    /// let resolved = handle.resolve_object(ObjectUuid::new_v4()).await?;
    /// assert_eq!(resolved, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_object(&self, object_uuid: ObjectUuid) -> Result<Option<ObjectId>, Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryObject(QueryObjectRequest {
                object_uuid,
                reply: rep_send,
                with_services: false,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        match rep_recv.await.map_err(|_| Error::ClientShutdown)? {
            Some((object_cookie, object_services)) => {
                debug_assert!(object_services.is_none());
                Ok(Some(ObjectId {
                    uuid: object_uuid,
                    cookie: object_cookie,
                }))
            }
            None => Ok(None),
        }
    }

    /// Queries the id and services of an object identified by a UUID.
    ///
    /// This returns the [`ObjectId`] as well as an [`ObjectServices`] stream of all
    /// [`ServiceId`s](ServiceId) of the object identified by `uuid`. If `uuid` does not name a
    /// valid object, then `None` is returned.
    ///
    /// Use [`resolve_object`](Handle::resolve_object) if you just need to resolve an [`ObjectUuid`]
    /// to an [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// use aldrin_client::{ObjectUuid, ServiceUuid};
    /// use futures::stream::StreamExt;
    /// use std::collections::HashSet;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    /// let service1 = object.create_service(ServiceUuid::new_v4(), 1).await?;
    /// let service2 = object.create_service(ServiceUuid::new_v4(), 1).await?;
    ///
    /// let (object_id, object_services) = handle
    ///     .query_object_services(object.id().uuid)
    ///     .await?
    ///     .unwrap();
    /// assert_eq!(object_id, object.id());
    ///
    /// let mut service_ids: HashSet<_> = object_services.collect().await;
    /// assert_eq!(service_ids.len(), 2);
    /// assert!(service_ids.remove(&service1.id()));
    /// assert!(service_ids.remove(&service2.id()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_object_services(
        &self,
        object_uuid: ObjectUuid,
    ) -> Result<Option<(ObjectId, ObjectServices)>, Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryObject(QueryObjectRequest {
                object_uuid,
                reply: rep_send,
                with_services: true,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        match rep_recv.await.map_err(|_| Error::ClientShutdown)? {
            Some((object_cookie, object_services)) => {
                let object_id = ObjectId {
                    uuid: object_uuid,
                    cookie: object_cookie,
                };
                let recv = object_services.unwrap();
                Ok(Some((object_id, ObjectServices { object_id, recv })))
            }
            None => Ok(None),
        }
    }

    /// Queries the version of a service.
    ///
    /// If `service_id` does not name a valid service, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// use aldrin_client::{ObjectUuid, ServiceUuid};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    /// let service = object.create_service(ServiceUuid::new_v4(), 2).await?;
    ///
    /// let version = handle.query_service_version(service.id()).await?;
    /// assert_eq!(version, Some(2));
    ///
    /// service.destroy().await?;
    /// let version = handle.query_service_version(service.id()).await?;
    /// assert_eq!(version, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_service_version(&self, service_id: ServiceId) -> Result<Option<u32>, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryServiceVersion(
                QueryServiceVersionRequest {
                    cookie: service_id.cookie,
                    reply,
                },
            ))
            .map_err(|_| Error::ClientShutdown)?;

        match recv.await.map_err(|_| Error::ClientShutdown)? {
            QueryServiceVersionResult::Ok(version) => Ok(Some(version)),
            QueryServiceVersionResult::InvalidService => Ok(None),
        }
    }

    /// Creates a channel and automatically claims the sender.
    ///
    /// When creating a channel, one of the two end must be claimed immediately. This function
    /// claims the sender. Use
    /// [`create_channel_with_claimed_receiver`](Self::create_channel_with_claimed_receiver) to
    /// claim the receiver instead.
    ///
    /// # Examples
    ///
    /// This example assumes that there are 2 clients, represented here by `handle1` and `handle2`.
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// use futures::StreamExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle1 = broker.add_client().await;
    /// # let handle2 = broker.add_client().await;
    /// // Client 1 creates the channel. It then unbinds the receiver and makes it available to
    /// // client 2. This will typically happen by returning it from a function call.
    /// let (sender, receiver) = handle1.create_channel_with_claimed_sender::<u32>().await?;
    /// let receiver = receiver.unbind();
    ///
    /// // Client 2 gets access to the receiver, and then binds and claims it.
    /// let mut receiver = receiver.claim(handle2.clone()).await?;
    ///
    /// // Meanwhile, client 1 waits for the receiver to be claimed.
    /// let mut sender = sender.established().await?;
    ///
    /// // The channel is now fully established and client 1 can send items to client 2.
    /// sender.send(1)?;
    /// sender.send(2)?;
    /// sender.send(3)?;
    ///
    /// // Client 1 will destroy (or drop) the channel when it has nothing to send anymore.
    /// sender.destroy().await?;
    ///
    /// // Client 2 receives all values in order. The Result in the return values can indicate
    /// // conversion errors when an item isn't a u32.
    /// assert_eq!(receiver.next().await, Some(Ok(1)));
    /// assert_eq!(receiver.next().await, Some(Ok(2)));
    /// assert_eq!(receiver.next().await, Some(Ok(3)));
    ///
    /// // Client 2 can observe that the sender has been destroyed by receiving None. It follows by
    /// // also destroying (or dropping) the receiver.
    /// assert_eq!(receiver.next().await, None);
    /// receiver.destroy().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_channel_with_claimed_sender<T>(
        &self,
    ) -> Result<(PendingSender<T>, UnclaimedReceiver<T>), Error>
    where
        T: IntoValue + FromValue,
    {
        todo!()
    }

    /// Creates a channel and automatically claims the receiver.
    ///
    /// When creating a channel, one of the two end must be claimed immediately. This function
    /// claims the receiver. Use
    /// [`create_channel_with_claimed_sender`](Self::create_channel_with_claimed_sender) to claim
    /// the sender instead.
    ///
    /// # Examples
    ///
    /// See [`create_channel_with_claimed_sender`](Self::create_channel_with_claimed_sender) for an
    /// example.
    pub async fn create_channel_with_claimed_receiver<T>(
        &self,
    ) -> Result<(UnclaimedSender<T>, PendingReceiver<T>), Error>
    where
        T: IntoValue + FromValue,
    {
        todo!()
    }

    pub(crate) async fn destroy_channel_end(
        &self,
        cookie: ChannelCookie,
        end: ChannelEnd,
        claimed: bool,
    ) -> Result<(), Error> {
        todo!()
    }

    pub(crate) fn destroy_channel_end_now(
        &self,
        cookie: ChannelCookie,
        end: ChannelEnd,
        claimed: bool,
    ) {
        todo!()
    }

    pub(crate) async fn claim_sender(&self, cookie: ChannelCookie) -> Result<SenderInner, Error> {
        todo!()
    }

    pub(crate) async fn claim_receiver(
        &self,
        cookie: ChannelCookie,
    ) -> Result<ReceiverInner, Error> {
        todo!()
    }

    pub(crate) fn send_item(&self, cookie: ChannelCookie, item: Value) -> Result<(), Error> {
        todo!()
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        self.send.unbounded_send(HandleRequest::HandleCloned).ok();
        Handle {
            send: self.send.clone(),
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.send.unbounded_send(HandleRequest::HandleDropped).ok();
    }
}

/// Future to await the result of a function call.
///
/// The future resolves to the type `Result<Result<T, E>, Error>`. The outer `Result<_, Error>`
/// represents the success or failure ([`Error`]) on the protocol and client library level. The
/// inner `Result<T, E>` represents the actual result of the function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingFunctionResult<T = Value, E = Value> {
    recv: oneshot::Receiver<CallFunctionResult>,
    service_id: ServiceId,
    function: u32,
    _res: PhantomData<fn() -> (T, E)>,
}

impl<T, E> PendingFunctionResult<T, E> {
    pub(crate) fn new(
        recv: oneshot::Receiver<CallFunctionResult>,
        service_id: ServiceId,
        function: u32,
    ) -> Self {
        PendingFunctionResult {
            recv,
            service_id,
            function,
            _res: PhantomData,
        }
    }
}

impl<T: FromValue, E: FromValue> Future for PendingFunctionResult<T, E> {
    type Output = Result<Result<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let res = match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(_)) => return Poll::Ready(Err(Error::ClientShutdown)),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(match res {
            CallFunctionResult::Ok(t) => match t.convert() {
                Ok(t) => Ok(Ok(t)),
                Err(e) => Err(InvalidFunctionResult {
                    service_id: self.service_id,
                    function: self.function,
                    result: Ok(e.0),
                }
                .into()),
            },
            CallFunctionResult::Err(e) => match e.convert() {
                Ok(e) => Ok(Err(e)),
                Err(e) => Err(InvalidFunctionResult {
                    service_id: self.service_id,
                    function: self.function,
                    result: Err(e.0),
                }
                .into()),
            },
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(self.service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(self.service_id, self.function))
            }
            CallFunctionResult::InvalidArgs => {
                Err(Error::InvalidArgs(self.service_id, self.function))
            }
        })
    }
}

impl<T, E> fmt::Debug for PendingFunctionResult<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingFunctionResult")
            .field("recv", &self.recv)
            .field("service_id", &self.service_id)
            .field("function", &self.function)
            .field("_res", &self._res)
            .finish()
    }
}

/// Future to await the result of an infallible function call.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingFunctionValue<T = Value>(PendingFunctionResult<T, Value>);

impl<T: FromValue> Future for PendingFunctionValue<T> {
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(res) => Poll::Ready(res?.map_err(|e| {
                InvalidFunctionResult {
                    service_id: self.0.service_id,
                    function: self.0.function,
                    result: Err(Some(e)),
                }
                .into()
            })),

            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> fmt::Debug for PendingFunctionValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PendingFunctionValue")
            .field(&self.0)
            .finish()
    }
}

/// Stream of service ids of an object.
///
/// This stream is created with [`query_object_services`](Handle::query_object_services).
#[derive(Debug)]
#[must_use = "streams do nothing unless you poll them"]
pub struct ObjectServices {
    object_id: ObjectId,
    recv: UnboundedReceiver<(ServiceUuid, ServiceCookie)>,
}

impl Stream for ObjectServices {
    type Item = ServiceId;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.recv)
            .poll_next(cx)
            .map(|ids| ids.map(|(uuid, cookie)| ServiceId::new(self.object_id, uuid, cookie)))
    }
}

impl FusedStream for ObjectServices {
    fn is_terminated(&self) -> bool {
        self.recv.is_terminated()
    }
}
