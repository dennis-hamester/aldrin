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
mod objects_created;
mod objects_destroyed;
mod serial_map;
mod service;
mod services_created;
mod services_destroyed;
mod transport;

use crate::proto::broker::*;
use crate::proto::client::*;
use crate::proto::{BrokerMessage, ClientMessage};
use event::Event;
use futures_channel::{mpsc, oneshot};
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serial_map::SerialMap;
use uuid::Uuid;

pub use builder::Builder;
pub use error::{ConnectError, Error, RunError};
pub use handle::Handle;
pub use object::Object;
pub use objects_created::ObjectsCreated;
pub use objects_destroyed::ObjectsDestroyed;
pub use service::Service;
pub use services_created::ServicesCreated;
pub use services_destroyed::ServicesDestroyed;
pub use transport::Transport;

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
    objects_created: SerialMap<mpsc::Sender<Uuid>>,
    objects_destroyed: SerialMap<mpsc::Sender<Uuid>>,
    create_service: SerialMap<oneshot::Sender<CreateServiceResult>>,
    destroy_service: SerialMap<oneshot::Sender<DestroyServiceResult>>,
    services_created: SerialMap<mpsc::Sender<(Uuid, Uuid)>>,
    services_destroyed: SerialMap<mpsc::Sender<(Uuid, Uuid)>>,
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

    async fn handle_message<E>(&mut self, msg: BrokerMessage) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        match msg {
            BrokerMessage::CreateObjectReply(re) => {
                if let Some(send) = self.create_object.remove(re.serial) {
                    send.send(re.result).ok();
                }

                Ok(())
            }

            BrokerMessage::DestroyObjectReply(re) => {
                if let Some(send) = self.destroy_object.remove(re.serial) {
                    send.send(re.result).ok();
                }

                Ok(())
            }

            BrokerMessage::ObjectCreatedEvent(ev) => self.object_created_event(ev).await,
            BrokerMessage::ObjectDestroyedEvent(ev) => self.object_destroyed_event(ev).await,

            BrokerMessage::CreateServiceReply(re) => {
                if let Some(send) = self.create_service.remove(re.serial) {
                    send.send(re.result).ok();
                }

                Ok(())
            }

            BrokerMessage::DestroyServiceReply(re) => {
                if let Some(send) = self.destroy_service.remove(re.serial) {
                    send.send(re.result).ok();
                }

                Ok(())
            }

            BrokerMessage::ServiceCreatedEvent(ev) => self.service_created_event(ev).await,
            BrokerMessage::ServiceDestroyedEvent(ev) => self.service_destroyed_event(ev).await,

            BrokerMessage::CallFunctionReply(_) => unimplemented!(),

            BrokerMessage::ConnectReply(_) => Err(RunError::UnexpectedMessageReceived(msg).into()),
        }
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
                if let Err(e) = send.send(object_created_event.id).await {
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
                if let Err(e) = send.send(object_created_event.id).await {
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
            self.t
                .send(ClientMessage::UnsubscribeObjectsCreated)
                .await?;
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
            if let Err(e) = send.send(object_destroyed_event.id).await {
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
            self.t
                .send(ClientMessage::UnsubscribeObjectsDestroyed)
                .await?;
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
                    .send((service_created_event.object_id, service_created_event.id))
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
                    .send((service_created_event.object_id, service_created_event.id))
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
            self.t
                .send(ClientMessage::UnsubscribeServicesCreated)
                .await?;
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
                    service_destroyed_event.object_id,
                    service_destroyed_event.id,
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
            self.t
                .send(ClientMessage::UnsubscribeServicesDestroyed)
                .await?;
        }

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
            Event::CreateService(object_id, id, reply) => {
                self.create_service(object_id, id, reply).await
            }
            Event::DestroyService(object_id, id, reply) => {
                self.destroy_service(object_id, id, reply).await
            }
            Event::SubscribeServicesCreated(sender, with_current) => {
                self.subscribe_services_created(sender, with_current).await
            }
            Event::SubscribeServicesDestroyed(sender) => {
                self.subscribe_services_destroyed(sender).await
            }

            // Handled in Client::run()
            Event::Shutdown => unreachable!(),
        }
    }

    async fn create_object<E>(
        &mut self,
        id: Uuid,
        reply: oneshot::Sender<CreateObjectResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.create_object.insert(reply);
        self.t
            .send(ClientMessage::CreateObject(CreateObject { serial, id }))
            .await
            .map_err(Into::into)
    }

    async fn destroy_object<E>(
        &mut self,
        id: Uuid,
        reply: oneshot::Sender<DestroyObjectResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_object.insert(reply);
        self.t
            .send(ClientMessage::DestroyObject(DestroyObject { serial, id }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_objects_created<E>(
        &mut self,
        sender: mpsc::Sender<Uuid>,
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
                .send(ClientMessage::SubscribeObjectsCreated(
                    SubscribeObjectsCreated { serial },
                ))
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn subscribe_objects_destroyed<E>(&mut self, sender: mpsc::Sender<Uuid>) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = self.objects_destroyed.is_empty();
        self.objects_destroyed.insert(sender);
        if send {
            self.t
                .send(ClientMessage::SubscribeObjectsDestroyed)
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    async fn create_service<E>(
        &mut self,
        object_id: Uuid,
        id: Uuid,
        reply: oneshot::Sender<CreateServiceResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.create_service.insert(reply);
        self.t
            .send(ClientMessage::CreateService(CreateService {
                serial,
                object_id,
                id,
            }))
            .await
            .map_err(Into::into)
    }

    async fn destroy_service<E>(
        &mut self,
        object_id: Uuid,
        id: Uuid,
        reply: oneshot::Sender<DestroyServiceResult>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let serial = self.destroy_service.insert(reply);
        self.t
            .send(ClientMessage::DestroyService(DestroyService {
                serial,
                object_id,
                id,
            }))
            .await
            .map_err(Into::into)
    }

    async fn subscribe_services_created<E>(
        &mut self,
        sender: mpsc::Sender<(Uuid, Uuid)>,
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
                .send(ClientMessage::SubscribeServicesCreated(
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
        sender: mpsc::Sender<(Uuid, Uuid)>,
    ) -> Result<(), E>
    where
        E: From<RunError> + From<T::Error>,
    {
        let send = self.services_destroyed.is_empty();
        self.services_destroyed.insert(sender);
        if send {
            self.t
                .send(ClientMessage::SubscribeServicesDestroyed)
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }
}
