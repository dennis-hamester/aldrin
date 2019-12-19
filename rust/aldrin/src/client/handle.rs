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

use super::{
    Error, Event, Object, ObjectProxy, ObjectsCreated, ObjectsDestroyed, Service, ServiceProxy,
    ServicesCreated, ServicesDestroyed,
};
use crate::proto::broker::*;
use futures_channel::mpsc::{channel, Sender};
use futures_channel::oneshot;
use futures_util::sink::SinkExt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Handle {
    send: Sender<Event>,
    event_fifo_size: usize,
}

impl Handle {
    pub(crate) fn new(send: Sender<Event>, event_fifo_size: usize) -> Self {
        Handle {
            send,
            event_fifo_size,
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.send.send(Event::Shutdown).await.map_err(Into::into)
    }

    pub async fn create_object(&mut self, id: Uuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::CreateObject(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CreateObjectResult::Ok => Ok(Object::new(id, self.clone())),
            CreateObjectResult::DuplicateId => Err(Error::DuplicateObject(id)),
        }
    }

    pub(crate) async fn destroy_object(&mut self, id: Uuid) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::DestroyObject(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) fn destroy_object_now(&mut self, id: Uuid) {
        let (res_send, _) = oneshot::channel();
        self.send.try_send(Event::DestroyObject(id, res_send)).ok();
    }

    pub async fn objects_created(&mut self, with_current: bool) -> Result<ObjectsCreated, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeObjectsCreated(ev_send, with_current))
            .await?;
        Ok(ObjectsCreated::new(ev_recv))
    }

    pub async fn objects_destroyed(&mut self) -> Result<ObjectsDestroyed, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeObjectsDestroyed(ev_send))
            .await?;
        Ok(ObjectsDestroyed::new(ev_recv))
    }

    pub(crate) async fn create_service(
        &mut self,
        object_id: Uuid,
        id: Uuid,
    ) -> Result<Service, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Event::CreateService(object_id, id, res_send))
            .await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CreateServiceResult::Ok => Ok(Service::new(object_id, id, self.clone())),
            CreateServiceResult::DuplicateId => Err(Error::DuplicateService(object_id, id)),
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject(object_id)),
            CreateServiceResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) async fn destroy_service(&mut self, object_id: Uuid, id: Uuid) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Event::DestroyService(object_id, id, res_send))
            .await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyServiceResult::Ok => Ok(()),
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(object_id, id)),
            DestroyServiceResult::InvalidObject => Err(Error::InvalidObject(object_id)),
            DestroyServiceResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) fn destroy_service_now(&mut self, object_id: Uuid, id: Uuid) {
        let (res_send, _) = oneshot::channel();
        self.send
            .try_send(Event::DestroyService(object_id, id, res_send))
            .ok();
    }

    pub async fn services_created(&mut self, with_current: bool) -> Result<ServicesCreated, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeServicesCreated(ev_send, with_current))
            .await?;
        Ok(ServicesCreated::new(ev_recv))
    }

    pub async fn services_destroyed(&mut self) -> Result<ServicesDestroyed, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeServicesDestroyed(ev_send))
            .await?;
        Ok(ServicesDestroyed::new(ev_recv))
    }

    pub fn bind_object_proxy(&self, id: Uuid) -> ObjectProxy {
        ObjectProxy::new(id, self.clone())
    }

    pub fn bind_service_proxy(&self, object_id: Uuid, service_id: Uuid) -> ServiceProxy {
        ServiceProxy::new(object_id, service_id, self.clone())
    }
}
