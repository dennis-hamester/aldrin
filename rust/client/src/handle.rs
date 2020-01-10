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
    Error, Event, Object, ObjectCookie, ObjectId, ObjectProxy, ObjectUuid, ObjectsCreated,
    ObjectsDestroyed, Service, ServiceCookie, ServiceId, ServiceProxy, ServiceUuid,
    ServicesCreated, ServicesDestroyed, SubscribeMode,
};
use aldrin_proto::*;
use futures_channel::mpsc::{channel, Sender};
use futures_channel::oneshot;
use futures_util::sink::SinkExt;

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

    pub async fn create_object(&mut self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::CreateObject(uuid, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CreateObjectResult::Ok(cookie) => Ok(Object::new(
                ObjectId::new(uuid, ObjectCookie(cookie)),
                self.clone(),
            )),
            CreateObjectResult::DuplicateObject => Err(Error::DuplicateObject(uuid)),
        }
    }

    pub(crate) async fn destroy_object(&mut self, id: ObjectId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::DestroyObject(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) fn destroy_object_now(&mut self, id: ObjectId) {
        let (res_send, _) = oneshot::channel();
        self.send.try_send(Event::DestroyObject(id, res_send)).ok();
    }

    pub async fn objects_created(&mut self, mode: SubscribeMode) -> Result<ObjectsCreated, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeObjectsCreated(ev_send, mode))
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
        object_id: ObjectId,
        uuid: ServiceUuid,
        fifo_size: usize,
    ) -> Result<Service, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Event::CreateService(object_id, uuid, fifo_size, res_send))
            .await?;
        let (res, recv) = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match res {
            CreateServiceResult::Ok(cookie) => Ok(Service::new(
                object_id,
                ServiceId::new(uuid, ServiceCookie(cookie)),
                self.clone(),
                recv.unwrap(),
            )),
            CreateServiceResult::DuplicateService => Err(Error::DuplicateService(object_id, uuid)),
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject(object_id)),
            CreateServiceResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) async fn destroy_service(
        &mut self,
        object_id: ObjectId,
        id: ServiceId,
    ) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::DestroyService(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyServiceResult::Ok => Ok(()),
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(object_id, id)),
            DestroyServiceResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub(crate) fn destroy_service_now(&mut self, id: ServiceId) {
        let (res_send, _) = oneshot::channel();
        self.send.try_send(Event::DestroyService(id, res_send)).ok();
    }

    pub async fn services_created(
        &mut self,
        mode: SubscribeMode,
    ) -> Result<ServicesCreated, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeServicesCreated(ev_send, mode))
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

    pub fn bind_object_proxy(&self, id: ObjectId) -> ObjectProxy {
        ObjectProxy::new(id, self.clone())
    }

    pub fn bind_service_proxy(&self, object_id: ObjectId, service_id: ServiceId) -> ServiceProxy {
        ServiceProxy::new(object_id, service_id, self.clone())
    }

    pub(crate) async fn call_function(
        &mut self,
        object_id: ObjectId,
        service_id: ServiceId,
        function: u32,
        args: Value,
    ) -> Result<Result<Value, Value>, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Event::CallFunction(service_id, function, args, res_send))
            .await?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CallFunctionResult::Ok(v) => Ok(Ok(v)),
            CallFunctionResult::Err(v) => Ok(Err(v)),
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(object_id, service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(object_id, service_id, function))
            }
            CallFunctionResult::InvalidArgs => {
                Err(Error::InvalidArgs(object_id, service_id, function))
            }
        }
    }

    pub(crate) async fn function_call_reply(
        &mut self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .send(Event::FunctionCallReply(serial, result))
            .await
            .map_err(Into::into)
    }

    pub(crate) fn abort_function_call_now(&mut self, serial: u32) {
        self.send
            .try_send(Event::FunctionCallReply(
                serial,
                CallFunctionResult::Aborted,
            ))
            .ok();
    }
}
