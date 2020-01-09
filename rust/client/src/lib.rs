// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod builder;
mod error;
mod event;
mod handle;
mod object;
mod object_proxy;
mod objects_created;
mod objects_destroyed;
mod serial_map;
mod service;
mod service_proxy;
mod services_created;
mod services_destroyed;

use aldrin_proto::*;
use aldrin_transport::Transport;
use event::Event;
use futures_channel::{mpsc, oneshot};
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serial_map::SerialMap;
use std::collections::HashMap;

pub use builder::Builder;
pub use error::{ConnectError, Error, RunError};
pub use handle::Handle;
pub use object::{Object, ObjectCookie, ObjectId, ObjectUuid};
pub use object_proxy::ObjectProxy;
pub use objects_created::ObjectsCreated;
pub use objects_destroyed::ObjectsDestroyed;
pub use service::{
    FunctionCall, FunctionCallReply, Service, ServiceCookie, ServiceId, ServiceUuid,
};
pub use service_proxy::ServiceProxy;
pub use services_created::ServicesCreated;
pub use services_destroyed::ServicesDestroyed;

#[derive(Debug)]
pub struct Client<T>
where
    T: Transport + Unpin,
{
    t: T,
    recv: mpsc::Receiver<Event>,
    handle: Option<Handle>,
    create_object: SerialMap<oneshot::Sender<CreateObjectResult>>,
    destroy_object: SerialMap<oneshot::Sender<DestroyObjectResult>>,
    objects_created: SerialMap<mpsc::Sender<ObjectId>>,
    objects_destroyed: SerialMap<mpsc::Sender<ObjectId>>,
    create_service: SerialMap<(
        usize,
        oneshot::Sender<(
            CreateServiceResult,
            Option<mpsc::Receiver<(u32, Value, u32)>>,
        )>,
    )>,
    destroy_service: SerialMap<(ServiceCookie, oneshot::Sender<DestroyServiceResult>)>,
    services_created: SerialMap<mpsc::Sender<(ObjectId, ServiceId)>>,
    services_destroyed: SerialMap<mpsc::Sender<(ObjectId, ServiceId)>>,
    function_calls: SerialMap<oneshot::Sender<CallFunctionResult>>,
    services: HashMap<ServiceCookie, mpsc::Sender<(u32, Value, u32)>>,
}

impl<T> Client<T>
where
    T: Transport + Unpin,
{
    pub fn builder(t: T) -> Builder<T> {
        Builder::new(t)
    }

    pub(crate) fn new(t: T, fifo_size: usize, event_fifo_size: usize) -> Self {
        let (send, recv) = mpsc::channel(fifo_size);
        Client {
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
        }
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
                Either::Right((Some(Event::Shutdown), _)) => return Ok(()),
                Either::Right((Some(ev), _)) => self.handle_event::<E>(ev).await?,
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
            Message::CreateServiceReply(re) => self.create_service_reply(re).await,
            Message::DestroyServiceReply(re) => self.destroy_service_reply(re).await,
            Message::ServiceCreatedEvent(ev) => self.service_created_event(ev).await,
            Message::ServiceDestroyedEvent(ev) => self.service_destroyed_event(ev).await,
            Message::CallFunction(ev) => self.function_call(ev).await,
            Message::CallFunctionReply(ev) => self.call_function_reply(ev).await,

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
            if let Some(send) = self.objects_created.get_mut(serial) {
                if let Err(e) = send
                    .send(ObjectId::new(
                        ObjectUuid(object_created_event.uuid),
                        ObjectCookie(object_created_event.cookie),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        self.objects_created.remove(serial);
                    } else if e.is_full() {
                        return Err(RunError::EventFifoOverflow.into());
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, send) in self.objects_created.iter_mut() {
                if let Err(e) = send
                    .send(ObjectId::new(
                        ObjectUuid(object_created_event.uuid),
                        ObjectCookie(object_created_event.cookie),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        remove.push(serial);
                    } else if e.is_full() {
                        return Err(RunError::EventFifoOverflow.into());
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
                } else if e.is_full() {
                    return Err(RunError::EventFifoOverflow.into());
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
        if let Some((cookie, send)) = self.destroy_service.remove(reply.serial) {
            if let DestroyServiceResult::Ok = reply.result {
                let contained = self.services.remove(&cookie);
                debug_assert!(contained.is_some());
            }

            send.send(reply.result).ok();
        }

        Ok(())
    }

    async fn service_created_event<E>(
        &mut self,
        service_created_event: ServiceCreatedEvent,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        if let Some(serial) = service_created_event.serial {
            if let Some(send) = self.services_created.get_mut(serial) {
                if let Err(e) = send
                    .send((
                        ObjectId::new(
                            ObjectUuid(service_created_event.object_uuid),
                            ObjectCookie(service_created_event.object_cookie),
                        ),
                        ServiceId::new(
                            ServiceUuid(service_created_event.uuid),
                            ServiceCookie(service_created_event.cookie),
                        ),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        self.services_created.remove(serial);
                    } else if e.is_full() {
                        return Err(RunError::EventFifoOverflow.into());
                    } else {
                        return Err(RunError::InternalError.into());
                    }
                }
            }
        } else {
            let mut remove = Vec::new();

            for (serial, send) in self.services_created.iter_mut() {
                if let Err(e) = send
                    .send((
                        ObjectId::new(
                            ObjectUuid(service_created_event.object_uuid),
                            ObjectCookie(service_created_event.object_cookie),
                        ),
                        ServiceId::new(
                            ServiceUuid(service_created_event.uuid),
                            ServiceCookie(service_created_event.cookie),
                        ),
                    ))
                    .await
                {
                    if e.is_disconnected() {
                        remove.push(serial);
                    } else if e.is_full() {
                        return Err(RunError::EventFifoOverflow.into());
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

    async fn service_destroyed_event<E>(
        &mut self,
        service_destroyed_event: ServiceDestroyedEvent,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let mut remove = Vec::new();

        for (serial, send) in self.services_destroyed.iter_mut() {
            if let Err(e) = send
                .send((
                    ObjectId::new(
                        ObjectUuid(service_destroyed_event.object_uuid),
                        ObjectCookie(service_destroyed_event.object_cookie),
                    ),
                    ServiceId::new(
                        ServiceUuid(service_destroyed_event.uuid),
                        ServiceCookie(service_destroyed_event.cookie),
                    ),
                ))
                .await
            {
                if e.is_disconnected() {
                    remove.push(serial);
                } else if e.is_full() {
                    return Err(RunError::EventFifoOverflow.into());
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

    async fn handle_event<E>(&mut self, ev: Event) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        match ev {
            Event::CreateObject(id, reply) => self.create_object(id, reply).await,
            Event::DestroyObject(id, reply) => self.destroy_object(id, reply).await,
            Event::SubscribeObjectsCreated(sender, with_current) => {
                self.subscribe_objects_created(sender, with_current).await
            }
            Event::SubscribeObjectsDestroyed(sender) => {
                self.subscribe_objects_destroyed(sender).await
            }
            Event::CreateService(object_id, id, fifo_size, reply) => {
                self.create_service(object_id, id, fifo_size, reply).await
            }
            Event::DestroyService(id, reply) => self.destroy_service(id, reply).await,
            Event::SubscribeServicesCreated(sender, with_current) => {
                self.subscribe_services_created(sender, with_current).await
            }
            Event::SubscribeServicesDestroyed(sender) => {
                self.subscribe_services_destroyed(sender).await
            }
            Event::CallFunction(service_id, function, args, reply) => {
                self.call_function(service_id, function, args, reply).await
            }
            Event::FunctionCallReply(serial, result) => {
                self.function_call_reply(serial, result).await
            }

            // Handled in Client::run()
            Event::Shutdown => unreachable!(),
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
        id: ObjectId,
        reply: oneshot::Sender<DestroyObjectResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_object.insert(reply);
        self.t
            .send(Message::DestroyObject(DestroyObject {
                serial,
                cookie: id.cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_objects_created<E>(
        &mut self,
        sender: mpsc::Sender<ObjectId>,
        with_current: bool,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = with_current || self.objects_created.is_empty();
        let serial = self.objects_created.insert(sender);
        if send {
            let serial = if with_current { Some(serial) } else { None };
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
        object_id: ObjectId,
        uuid: ServiceUuid,
        fifo_size: usize,
        reply: oneshot::Sender<(
            CreateServiceResult,
            Option<mpsc::Receiver<(u32, Value, u32)>>,
        )>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.create_service.insert((fifo_size, reply));
        self.t
            .send(Message::CreateService(CreateService {
                serial,
                object_cookie: object_id.cookie.0,
                uuid: uuid.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn destroy_service<E>(
        &mut self,
        id: ServiceId,
        reply: oneshot::Sender<DestroyServiceResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_service.insert((id.cookie, reply));
        self.t
            .send(Message::DestroyService(DestroyService {
                serial,
                cookie: id.cookie.0,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_services_created<E>(
        &mut self,
        sender: mpsc::Sender<(ObjectId, ServiceId)>,
        with_current: bool,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = with_current || self.services_created.is_empty();
        let serial = self.services_created.insert(sender);
        if send {
            let serial = if with_current { Some(serial) } else { None };
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
        sender: mpsc::Sender<(ObjectId, ServiceId)>,
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
        service_id: ServiceId,
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
                service_cookie: service_id.cookie.0,
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
}
