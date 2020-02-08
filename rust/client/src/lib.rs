mod error;
mod events;
mod handle;
mod object;
mod objects_created;
mod objects_destroyed;
mod request;
mod serial_map;
mod service;
mod services_created;
mod services_destroyed;

#[doc(hidden)]
pub mod codegen {
    pub use aldrin_proto;
    pub use futures_core;
    pub use uuid;
}

use aldrin_proto::*;
use events::{EventsId, EventsRequest};
use futures_channel::{mpsc, oneshot};
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use request::{EmitEventRequest, Request, SubscribeEventRequest, UnsubscribeEventRequest};
use serial_map::SerialMap;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

pub use error::{ConnectError, Error, RunError};
pub use events::{Event, Events};
pub use handle::Handle;
pub use object::{Object, ObjectCookie, ObjectId, ObjectUuid};
pub use objects_created::ObjectsCreated;
pub use objects_destroyed::ObjectsDestroyed;
pub use service::{
    FunctionCall, FunctionCallReply, Service, ServiceCookie, ServiceId, ServiceUuid,
};
pub use services_created::ServicesCreated;
pub use services_destroyed::ServicesDestroyed;

type FunctionCallReceiver = mpsc::Receiver<(u32, Value, u32)>;
type CreateServiceReplySender =
    oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>;
type Subscriptions = HashMap<u32, HashMap<EventsId, mpsc::Sender<EventsRequest>>>;

#[derive(Debug)]
pub struct Client<T>
where
    T: Transport + Unpin,
{
    t: T,
    recv: mpsc::Receiver<Request>,
    handle: Option<Handle>,
    create_object: SerialMap<oneshot::Sender<CreateObjectResult>>,
    destroy_object: SerialMap<oneshot::Sender<DestroyObjectResult>>,
    objects_created: SerialMap<(mpsc::Sender<ObjectId>, SubscribeMode)>,
    objects_destroyed: SerialMap<mpsc::Sender<ObjectId>>,
    create_service: SerialMap<(usize, CreateServiceReplySender)>,
    destroy_service: SerialMap<(ServiceCookie, oneshot::Sender<DestroyServiceResult>)>,
    services_created: SerialMap<(mpsc::Sender<ServiceId>, SubscribeMode)>,
    services_destroyed: SerialMap<mpsc::Sender<ServiceId>>,
    function_calls: SerialMap<oneshot::Sender<CallFunctionResult>>,
    services: HashMap<ServiceCookie, mpsc::Sender<(u32, Value, u32)>>,
    subscribe_event: SerialMap<(
        EventsId,
        ServiceCookie,
        u32,
        oneshot::Sender<SubscribeEventResult>,
    )>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
    broker_subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
}

impl<T> Client<T>
where
    T: Transport + Unpin,
{
    pub async fn connect<E>(mut t: T, fifo_size: usize, event_fifo_size: usize) -> Result<Self, E>
    where
        E: From<ConnectError> + From<T::Error>,
    {
        t.send(Message::Connect(Connect { version: VERSION }))
            .await?;

        match t.next().await.ok_or(ConnectError::UnexpectedEof)?? {
            Message::ConnectReply(ConnectReply::Ok) => {}
            Message::ConnectReply(ConnectReply::VersionMismatch(v)) => {
                return Err(ConnectError::VersionMismatch(v).into())
            }
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg).into()),
        }

        let (send, recv) = mpsc::channel(fifo_size);
        Ok(Client {
            t,
            recv,
            handle: Some(Handle::new(send, event_fifo_size)),
            create_object: SerialMap::new(),
            destroy_object: SerialMap::new(),
            objects_created: SerialMap::new(),
            objects_destroyed: SerialMap::new(),
            create_service: SerialMap::new(),
            destroy_service: SerialMap::new(),
            services_created: SerialMap::new(),
            services_destroyed: SerialMap::new(),
            function_calls: SerialMap::new(),
            services: HashMap::new(),
            subscribe_event: SerialMap::new(),
            subscriptions: HashMap::new(),
            broker_subscriptions: HashMap::new(),
        })
    }

    pub fn handle(&self) -> &Handle {
        self.handle.as_ref().unwrap()
    }

    pub async fn run<E>(mut self) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        self.handle.take().unwrap();

        loop {
            match select(self.t.next(), self.recv.next()).await {
                Either::Left((Some(Ok(msg)), _)) => self.handle_message::<E>(msg).await?,
                Either::Left((Some(Err(e)), _)) => return Err(e.into()),
                Either::Left((None, _)) => return Ok(()),
                Either::Right((Some(Request::Shutdown), _)) => return Ok(()),
                Either::Right((Some(req), _)) => self.handle_request::<E>(req).await?,
                Either::Right((None, _)) => return Ok(()),
            }
        }
    }

    async fn handle_message<E>(&mut self, msg: Message) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        match msg {
            Message::CreateObjectReply(re) => self.create_object_reply(re).await,
            Message::DestroyObjectReply(re) => self.destroy_object_reply(re).await,
            Message::ObjectCreatedEvent(ev) => self.object_created_event(ev).await,
            Message::ObjectDestroyedEvent(ev) => self.object_destroyed_event(ev).await,
            Message::SubscribeObjectsCreatedReply(re) => {
                self.subscribe_objects_created_reply(re).await
            }
            Message::CreateServiceReply(re) => self.create_service_reply(re).await,
            Message::DestroyServiceReply(re) => self.destroy_service_reply(re).await,
            Message::ServiceCreatedEvent(ev) => self.service_created_event(ev).await,
            Message::ServiceDestroyedEvent(ev) => self.service_destroyed_event(ev).await,
            Message::SubscribeServicesCreatedReply(re) => {
                self.subscribe_services_created_reply(re).await
            }
            Message::CallFunction(ev) => self.function_call(ev).await,
            Message::CallFunctionReply(ev) => self.call_function_reply(ev).await,
            Message::SubscribeEvent(ev) => self.event_subscribed(ev).await,
            Message::SubscribeEventReply(re) => self.subscribe_event_reply(re).await,
            Message::UnsubscribeEvent(ev) => self.event_unsubscribed(ev).await,
            Message::EmitEvent(ev) => self.event_emitted(ev).await,

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObject(_)
            | Message::SubscribeObjectsCreated(_)
            | Message::UnsubscribeObjectsCreated
            | Message::DestroyObject(_)
            | Message::SubscribeObjectsDestroyed
            | Message::UnsubscribeObjectsDestroyed
            | Message::CreateService(_)
            | Message::SubscribeServicesCreated(_)
            | Message::UnsubscribeServicesCreated
            | Message::DestroyService(_)
            | Message::SubscribeServicesDestroyed
            | Message::UnsubscribeServicesDestroyed => {
                Err(RunError::UnexpectedMessageReceived(msg).into())
            }
        }
    }

    async fn create_object_reply<E>(&mut self, reply: CreateObjectReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some(send) = self.create_object.remove(reply.serial) {
            send.send(reply.result).ok();
        }

        Ok(())
    }

    async fn destroy_object_reply<E>(&mut self, reply: DestroyObjectReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some(send) = self.destroy_object.remove(reply.serial) {
            send.send(reply.result).ok();
        }

        Ok(())
    }

    async fn object_created_event<E>(
        &mut self,
        object_created_event: ObjectCreatedEvent,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some(serial) = object_created_event.serial {
            if let Some((send, _)) = self.objects_created.get_mut(serial) {
                if let Err(e) = send
                    .send(ObjectId::new(
                        ObjectUuid(object_created_event.uuid),
                        ObjectCookie(object_created_event.cookie),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        self.objects_created.remove(serial);
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, (send, _)) in self.objects_created.iter_mut() {
                if let Err(e) = send
                    .send(ObjectId::new(
                        ObjectUuid(object_created_event.uuid),
                        ObjectCookie(object_created_event.cookie),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        remove.push(serial);
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }

            for serial in remove {
                self.objects_created.remove(serial);
            }
        }

        if self.objects_created.is_empty() {
            self.t.send(Message::UnsubscribeObjectsCreated).await?;
        }

        Ok(())
    }

    async fn object_destroyed_event<E>(
        &mut self,
        object_destroyed_event: ObjectDestroyedEvent,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let mut remove = Vec::new();

        for (serial, send) in self.objects_destroyed.iter_mut() {
            if let Err(e) = send
                .send(ObjectId::new(
                    ObjectUuid(object_destroyed_event.uuid),
                    ObjectCookie(object_destroyed_event.cookie),
                ))
                .await
            {
                if e.is_disconnected() {
                    remove.push(serial);
                } else {
                    return Err(RunError::InternalError.into());
                }
            }
        }

        for serial in remove {
            self.objects_destroyed.remove(serial);
        }

        if self.objects_destroyed.is_empty() {
            self.t.send(Message::UnsubscribeObjectsDestroyed).await?;
        }

        Ok(())
    }

    async fn subscribe_objects_created_reply<E>(
        &mut self,
        reply: SubscribeObjectsCreatedReply,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some((_, SubscribeMode::CurrentOnly)) = self.objects_created.get_mut(reply.serial) {
            self.objects_created.remove(reply.serial);
        }

        Ok(())
    }

    async fn create_service_reply<E>(&mut self, reply: CreateServiceReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some((fifo_size, rep_send)) = self.create_service.remove(reply.serial) {
            let recv = if let CreateServiceResult::Ok(cookie) = reply.result {
                let (send, recv) = mpsc::channel(fifo_size);
                let dup = self.services.insert(ServiceCookie(cookie), send);
                debug_assert!(dup.is_none());
                Some(recv)
            } else {
                None
            };

            rep_send.send((reply.result, recv)).ok();
        }

        Ok(())
    }

    async fn destroy_service_reply<E>(&mut self, reply: DestroyServiceReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let (cookie, send) = match self.destroy_service.remove(reply.serial) {
            Some(data) => data,
            None => return Ok(()),
        };

        if let DestroyServiceResult::Ok = reply.result {
            let contained = self.services.remove(&cookie);
            debug_assert!(contained.is_some());
            self.broker_subscriptions.remove(&cookie);
        }

        send.send(reply.result).ok();
        Ok(())
    }

    async fn service_created_event<E>(&mut self, ev: ServiceCreatedEvent) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let service_id = ServiceId::new(
            ObjectId::new(ObjectUuid(ev.object_uuid), ObjectCookie(ev.object_cookie)),
            ServiceUuid(ev.uuid),
            ServiceCookie(ev.cookie),
        );

        if let Some(serial) = ev.serial {
            if let Some((send, _)) = self.services_created.get_mut(serial) {
                if let Err(e) = send.send(service_id).await {
                    if e.is_disconnected() {
                        self.services_created.remove(serial);
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, (send, _)) in self.services_created.iter_mut() {
                if let Err(e) = send.send(service_id).await {
                    if e.is_disconnected() {
                        remove.push(serial);
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }

            for serial in remove {
                self.services_created.remove(serial);
            }
        }

        if self.services_created.is_empty() {
            self.t.send(Message::UnsubscribeServicesCreated).await?;
        }

        Ok(())
    }

    async fn service_destroyed_event<E>(&mut self, ev: ServiceDestroyedEvent) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let mut remove = Vec::new();
        let service_cookie = ServiceCookie(ev.cookie);
        let service_id = ServiceId::new(
            ObjectId::new(ObjectUuid(ev.object_uuid), ObjectCookie(ev.object_cookie)),
            ServiceUuid(ev.uuid),
            service_cookie,
        );

        // A ServiceDestroyedEvent can also be sent, when we have active subscriptions on a
        // service. If that is the sole reason for this event, then make sure not to send
        // UnsubscribeServicesDestroyed below.
        if !self.services_destroyed.is_empty() {
            for (serial, send) in self.services_destroyed.iter_mut() {
                if let Err(e) = send.send(service_id).await {
                    if e.is_disconnected() {
                        remove.push(serial);
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }

            for serial in remove {
                self.services_destroyed.remove(serial);
            }

            if self.services_destroyed.is_empty() {
                self.t.send(Message::UnsubscribeServicesDestroyed).await?;
            }
        }

        self.broker_subscriptions.remove(&service_cookie);

        if let Some(ids) = self.subscriptions.remove(&service_cookie) {
            let mut dups = HashSet::new();
            for (_, events_ids) in ids {
                for (events_id, mut sender) in events_ids {
                    if dups.insert(events_id) {
                        // Should we close the channel in case of send errors?
                        sender
                            .send(EventsRequest::ServiceDestroyed(service_cookie))
                            .await
                            .ok();
                    }
                }
            }
        }

        Ok(())
    }

    async fn subscribe_services_created_reply<E>(
        &mut self,
        reply: SubscribeServicesCreatedReply,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some((_, SubscribeMode::CurrentOnly)) = self.services_created.get_mut(reply.serial) {
            self.services_created.remove(reply.serial);
        }

        Ok(())
    }

    async fn function_call<E>(&mut self, call_function: CallFunction) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let cookie = ServiceCookie(call_function.service_cookie);
        let send = self.services.get_mut(&cookie).expect("inconsistent state");

        if let Err(e) = send
            .send((
                call_function.function,
                call_function.args,
                call_function.serial,
            ))
            .await
        {
            if e.is_disconnected() {
                self.t
                    .send(Message::CallFunctionReply(CallFunctionReply {
                        serial: call_function.serial,
                        result: CallFunctionResult::InvalidService,
                    }))
                    .await
                    .map_err(Into::into)
            } else {
                Err(RunError::InternalError.into())
            }
        } else {
            Ok(())
        }
    }

    async fn call_function_reply<E>(&mut self, ev: CallFunctionReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = self
            .function_calls
            .remove(ev.serial)
            .expect("inconsistent state");
        send.send(ev.result).ok();
        Ok(())
    }

    async fn event_subscribed<E>(&mut self, ev: SubscribeEvent) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        self.broker_subscriptions
            .entry(ServiceCookie(ev.service_cookie))
            .or_default()
            .insert(ev.event);
        Ok(())
    }

    async fn subscribe_event_reply<E>(&mut self, reply: SubscribeEventReply) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let (events_id, service_cookie, id, rep_send) =
            match self.subscribe_event.remove(reply.serial) {
                Some(req) => req,
                None => return Ok(()),
            };

        // || would short-circuit and changing the order would move reply.result out.
        let mut err = reply.result.is_err();
        err |= rep_send.send(reply.result).is_err();
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

    async fn event_unsubscribed<E>(&mut self, ev: UnsubscribeEvent) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let service_cookie = ServiceCookie(ev.service_cookie);
        let mut subs = match self.broker_subscriptions.entry(service_cookie) {
            Entry::Occupied(subs) => subs,
            Entry::Vacant(_) => return Ok(()),
        };

        subs.get_mut().remove(&ev.event);
        if subs.get().is_empty() {
            subs.remove();
        }
        Ok(())
    }

    async fn event_emitted<E>(&mut self, ev: EmitEvent) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let service_cookie = ServiceCookie(ev.service_cookie);
        let senders = match self
            .subscriptions
            .get_mut(&service_cookie)
            .and_then(|s| s.get_mut(&ev.event))
        {
            Some(senders) => senders,
            None => return Ok(()),
        };

        for sender in senders.values_mut() {
            // Should we close the channel in case of send errors?
            sender
                .send(EventsRequest::EmitEvent(
                    service_cookie,
                    ev.event,
                    ev.args.clone(),
                ))
                .await
                .ok();
        }

        Ok(())
    }

    async fn handle_request<E>(&mut self, req: Request) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        match req {
            Request::CreateObject(uuid, reply) => self.create_object(uuid, reply).await,
            Request::DestroyObject(cookie, reply) => self.destroy_object(cookie, reply).await,
            Request::SubscribeObjectsCreated(sender, mode) => {
                self.subscribe_objects_created(sender, mode).await
            }
            Request::SubscribeObjectsDestroyed(sender) => {
                self.subscribe_objects_destroyed(sender).await
            }
            Request::CreateService(object_cookie, service_uuid, fifo_size, reply) => {
                self.create_service(object_cookie, service_uuid, fifo_size, reply)
                    .await
            }
            Request::DestroyService(cookie, reply) => self.destroy_service(cookie, reply).await,
            Request::SubscribeServicesCreated(sender, mode) => {
                self.subscribe_services_created(sender, mode).await
            }
            Request::SubscribeServicesDestroyed(sender) => {
                self.subscribe_services_destroyed(sender).await
            }
            Request::CallFunction(service_cookie, function, args, reply) => {
                self.call_function(service_cookie, function, args, reply)
                    .await
            }
            Request::FunctionCallReply(serial, result) => {
                self.function_call_reply(serial, result).await
            }
            Request::SubscribeEvent(req) => self.subscribe_event(req).await,
            Request::UnsubscribeEvent(req) => self.unsubscribe_event(req).await,
            Request::EmitEvent(req) => self.emit_event(req).await,

            // Handled in Client::run()
            Request::Shutdown => unreachable!(),
        }
    }

    async fn create_object<E>(
        &mut self,
        uuid: ObjectUuid,
        reply: oneshot::Sender<CreateObjectResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.create_object.insert(reply);
        self.t
            .send(Message::CreateObject(CreateObject {
                serial,
                uuid: uuid.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn destroy_object<E>(
        &mut self,
        cookie: ObjectCookie,
        reply: oneshot::Sender<DestroyObjectResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_object.insert(reply);
        self.t
            .send(Message::DestroyObject(DestroyObject {
                serial,
                cookie: cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_objects_created<E>(
        &mut self,
        sender: mpsc::Sender<ObjectId>,
        mode: SubscribeMode,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let is_empty = self.objects_created.is_empty();
        let serial = self.objects_created.insert((sender, mode));
        let (send, serial) = match mode {
            SubscribeMode::All | SubscribeMode::CurrentOnly => (true, Some(serial)),
            SubscribeMode::NewOnly => (is_empty, None),
        };

        if send {
            self.t
                .send(Message::SubscribeObjectsCreated(SubscribeObjectsCreated {
                    serial,
                }))
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn subscribe_objects_destroyed<E>(
        &mut self,
        sender: mpsc::Sender<ObjectId>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = self.objects_destroyed.is_empty();
        self.objects_destroyed.insert(sender);
        if send {
            self.t
                .send(Message::SubscribeObjectsDestroyed)
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn create_service<E>(
        &mut self,
        object_cookie: ObjectCookie,
        service_uuid: ServiceUuid,
        fifo_size: usize,
        reply: oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.create_service.insert((fifo_size, reply));
        self.t
            .send(Message::CreateService(CreateService {
                serial,
                object_cookie: object_cookie.0,
                uuid: service_uuid.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn destroy_service<E>(
        &mut self,
        cookie: ServiceCookie,
        reply: oneshot::Sender<DestroyServiceResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_service.insert((cookie, reply));
        self.t
            .send(Message::DestroyService(DestroyService {
                serial,
                cookie: cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_services_created<E>(
        &mut self,
        sender: mpsc::Sender<ServiceId>,
        mode: SubscribeMode,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let is_empty = self.services_created.is_empty();
        let serial = self.services_created.insert((sender, mode));
        let (send, serial) = match mode {
            SubscribeMode::All | SubscribeMode::CurrentOnly => (true, Some(serial)),
            SubscribeMode::NewOnly => (is_empty, None),
        };

        if send {
            self.t
                .send(Message::SubscribeServicesCreated(
                    SubscribeServicesCreated { serial },
                ))
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn subscribe_services_destroyed<E>(
        &mut self,
        sender: mpsc::Sender<ServiceId>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = self.services_destroyed.is_empty();
        self.services_destroyed.insert(sender);
        if send {
            self.t
                .send(Message::SubscribeServicesDestroyed)
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn call_function<E>(
        &mut self,
        service_cookie: ServiceCookie,
        function: u32,
        args: Value,
        reply: oneshot::Sender<CallFunctionResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.function_calls.insert(reply);
        self.t
            .send(Message::CallFunction(CallFunction {
                serial,
                service_cookie: service_cookie.0,
                function,
                args,
            }))
            .await
            .map_err(Into::into)
    }

    async fn function_call_reply<E>(
        &mut self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        self.t
            .send(Message::CallFunctionReply(CallFunctionReply {
                serial,
                result,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_event<E>(&mut self, req: SubscribeEventRequest) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
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
                .send(Message::SubscribeEvent(SubscribeEvent {
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

    async fn unsubscribe_event<E>(&mut self, req: UnsubscribeEventRequest) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
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
            .send(Message::UnsubscribeEvent(UnsubscribeEvent {
                service_cookie: req.service_cookie.0,
                event: req.id,
            }))
            .await
            .map_err(Into::into)
    }

    async fn emit_event<E>(&mut self, req: EmitEventRequest) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        self.t
            .send(Message::EmitEvent(EmitEvent {
                service_cookie: req.service_cookie.0,
                event: req.event,
                args: req.args,
            }))
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SubscribeMode {
    All,
    CurrentOnly,
    NewOnly,
}
