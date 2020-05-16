use super::{
    EmitEventRequest, Error, Events, EventsId, EventsRequest, Object, ObjectCookie, ObjectId,
    ObjectUuid, Objects, Request, Service, ServiceCookie, ServiceId, ServiceUuid, ServicesCreated,
    ServicesDestroyed, SubscribeEventRequest, SubscribeMode, UnsubscribeEventRequest,
};
use aldrin_proto::*;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_channel::oneshot;

#[derive(Debug, Clone)]
pub struct Handle {
    send: UnboundedSender<Request>,
}

impl Handle {
    pub(crate) fn new(send: UnboundedSender<Request>) -> Self {
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
    pub fn shutdown(&self) {
        self.send.unbounded_send(Request::Shutdown).ok();
    }

    pub async fn create_object(&self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::CreateObject(uuid, res_send))
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

    pub(crate) async fn destroy_object(&self, id: ObjectId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyObject(id.cookie, res_send))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_object_now(&self, cookie: ObjectCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyObject(cookie, res_send))
            .ok();
    }

    pub fn objects(&self, mode: SubscribeMode) -> Result<Objects, Error> {
        let (ev_send, ev_recv) = unbounded();
        self.send
            .unbounded_send(Request::SubscribeObjects(ev_send, mode))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(Objects::new(ev_recv))
    }

    pub(crate) async fn create_service(
        &self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
    ) -> Result<Service, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::CreateService(
                object_id.cookie,
                service_uuid,
                res_send,
            ))
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

    pub(crate) async fn destroy_service(&self, id: ServiceId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyService(id.cookie, res_send))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyServiceResult::Ok => Ok(()),
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(id)),
            DestroyServiceResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_service_now(&self, cookie: ServiceCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyService(cookie, res_send))
            .ok();
    }

    pub fn services_created(&self, mode: SubscribeMode) -> Result<ServicesCreated, Error> {
        let (ev_send, ev_recv) = unbounded();
        self.send
            .unbounded_send(Request::SubscribeServicesCreated(ev_send, mode))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ServicesCreated::new(ev_recv))
    }

    pub fn services_destroyed(&self) -> Result<ServicesDestroyed, Error> {
        let (ev_send, ev_recv) = unbounded();
        self.send
            .unbounded_send(Request::SubscribeServicesDestroyed(ev_send))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(ServicesDestroyed::new(ev_recv))
    }

    pub async fn call_function(
        &self,
        service_id: ServiceId,
        function: u32,
        args: Value,
    ) -> Result<Result<Value, Value>, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::CallFunction(
                service_id.cookie,
                function,
                args,
                res_send,
            ))
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

    pub(crate) fn function_call_reply(
        &self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(Request::FunctionCallReply(serial, result))
            .map_err(|_| Error::ClientShutdown)
    }

    pub(crate) fn abort_function_call_now(&self, serial: u32) {
        self.send
            .unbounded_send(Request::FunctionCallReply(
                serial,
                CallFunctionResult::Aborted,
            ))
            .ok();
    }

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
            .unbounded_send(Request::SubscribeEvent(SubscribeEventRequest {
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
            .unbounded_send(Request::UnsubscribeEvent(UnsubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    pub async fn emit_event(
        &self,
        service_id: ServiceId,
        event: u32,
        args: Value,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(Request::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                args,
            }))
            .map_err(|_| Error::ClientShutdown)
    }
}
