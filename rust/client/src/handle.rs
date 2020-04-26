use super::{
    EmitEventRequest, Error, Events, EventsId, EventsRequest, Object, ObjectCookie, ObjectId,
    ObjectUuid, ObjectsCreated, ObjectsDestroyed, Request, Service, ServiceCookie, ServiceId,
    ServiceUuid, ServicesCreated, ServicesDestroyed, SubscribeEventRequest, SubscribeMode,
    UnsubscribeEventRequest,
};
use aldrin_proto::*;
use futures_channel::mpsc::{channel, Sender};
use futures_channel::oneshot;
use futures_util::sink::SinkExt;

#[derive(Debug, Clone)]
pub struct Handle {
    send: Sender<Request>,
}

impl Handle {
    pub(crate) fn new(send: Sender<Request>) -> Self {
        Handle { send }
    }

    /// Shuts down the client.
    ///
    /// Client shutdown happens asynchronously, in the sense that when this function returns, the
    /// client has only been requested to shut down and not yet necessarily done so. As soon as
    /// [`Client::run`](super::Client::run) returns, it has fully shut down.
    ///
    /// If the client has already shut down (due to any reason), this function will not treat that
    /// as an error. This is different than most other functions, which would return
    /// [`Error::ClientShutdown`] instead.
    pub async fn shutdown(&mut self) {
        self.send.send(Request::Shutdown).await.ok();
    }

    pub async fn create_object(&mut self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Request::CreateObject(uuid, res_send))
            .await
            .map_err(|_| Error::ClientShutdown)?;
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
        self.send
            .send(Request::DestroyObject(id.cookie, res_send))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_object_now(&mut self, cookie: ObjectCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .try_send(Request::DestroyObject(cookie, res_send))
            .ok();
    }

    pub async fn objects_created(
        &mut self,
        mode: SubscribeMode,
        fifo_size: usize,
    ) -> Result<ObjectsCreated, Error> {
        let (ev_send, ev_recv) = channel(fifo_size);
        self.send
            .send(Request::SubscribeObjectsCreated(ev_send, mode))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ObjectsCreated::new(ev_recv))
    }

    pub async fn objects_destroyed(&mut self, fifo_size: usize) -> Result<ObjectsDestroyed, Error> {
        let (ev_send, ev_recv) = channel(fifo_size);
        self.send
            .send(Request::SubscribeObjectsDestroyed(ev_send))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ObjectsDestroyed::new(ev_recv))
    }

    pub(crate) async fn create_service(
        &mut self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
        fifo_size: usize,
    ) -> Result<Service, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Request::CreateService(
                object_id.cookie,
                service_uuid,
                fifo_size,
                res_send,
            ))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        let (res, recv) = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match res {
            CreateServiceResult::Ok(cookie) => Ok(Service::new(
                ServiceId::new(object_id, service_uuid, ServiceCookie(cookie)),
                self.clone(),
                recv.unwrap(),
            )),
            CreateServiceResult::DuplicateService => {
                Err(Error::DuplicateService(object_id, service_uuid))
            }
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject(object_id)),
            CreateServiceResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) async fn destroy_service(&mut self, id: ServiceId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Request::DestroyService(id.cookie, res_send))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyServiceResult::Ok => Ok(()),
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(id)),
            DestroyServiceResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_service_now(&mut self, cookie: ServiceCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .try_send(Request::DestroyService(cookie, res_send))
            .ok();
    }

    pub async fn services_created(
        &mut self,
        mode: SubscribeMode,
        fifo_size: usize,
    ) -> Result<ServicesCreated, Error> {
        let (ev_send, ev_recv) = channel(fifo_size);
        self.send
            .send(Request::SubscribeServicesCreated(ev_send, mode))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ServicesCreated::new(ev_recv))
    }

    pub async fn services_destroyed(
        &mut self,
        fifo_size: usize,
    ) -> Result<ServicesDestroyed, Error> {
        let (ev_send, ev_recv) = channel(fifo_size);
        self.send
            .send(Request::SubscribeServicesDestroyed(ev_send))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ServicesDestroyed::new(ev_recv))
    }

    pub async fn call_function(
        &mut self,
        service_id: ServiceId,
        function: u32,
        args: Value,
    ) -> Result<Result<Value, Value>, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .send(Request::CallFunction(
                service_id.cookie,
                function,
                args,
                res_send,
            ))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CallFunctionResult::Ok(v) => Ok(Ok(v)),
            CallFunctionResult::Err(v) => Ok(Err(v)),
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(service_id, function))
            }
            CallFunctionResult::InvalidArgs => Err(Error::InvalidArgs(service_id, function)),
        }
    }

    pub(crate) async fn function_call_reply(
        &mut self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .send(Request::FunctionCallReply(serial, result))
            .await
            .map_err(|_| Error::ClientShutdown)
    }

    pub(crate) fn abort_function_call_now(&mut self, serial: u32) {
        self.send
            .try_send(Request::FunctionCallReply(
                serial,
                CallFunctionResult::Aborted,
            ))
            .ok();
    }

    pub fn events(&self, fifo_size: usize) -> Events {
        Events::new(self.clone(), fifo_size)
    }

    pub(crate) async fn subscribe_event(
        &mut self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
        sender: Sender<EventsRequest>,
    ) -> Result<(), Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .send(Request::SubscribeEvent(SubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
                sender,
                reply: rep_send,
            }))
            .await
            .map_err(|_| Error::ClientShutdown)?;
        let reply = rep_recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            SubscribeEventResult::Ok => Ok(()),
            SubscribeEventResult::InvalidService => Err(Error::InvalidService(service_id)),
        }
    }

    pub(crate) async fn unsubscribe_event(
        &mut self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
    ) -> Result<(), Error> {
        self.send
            .send(Request::UnsubscribeEvent(UnsubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .await
            .map_err(|_| Error::ClientShutdown)
    }

    pub async fn emit_event(
        &mut self,
        service_id: ServiceId,
        event: u32,
        args: Value,
    ) -> Result<(), Error> {
        self.send
            .send(Request::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                args,
            }))
            .await
            .map_err(|_| Error::ClientShutdown)
    }
}
