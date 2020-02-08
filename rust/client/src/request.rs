use super::{
    EventsId, EventsRequest, FunctionCallReceiver, ObjectCookie, ObjectId, ObjectUuid,
    ServiceCookie, ServiceId, ServiceUuid, SubscribeMode,
};
use aldrin_proto::*;
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(crate) enum Request {
    Shutdown,
    CreateObject(ObjectUuid, oneshot::Sender<CreateObjectResult>),
    DestroyObject(ObjectCookie, oneshot::Sender<DestroyObjectResult>),
    SubscribeObjectsCreated(mpsc::Sender<ObjectId>, SubscribeMode),
    SubscribeObjectsDestroyed(mpsc::Sender<ObjectId>),
    CreateService(
        ObjectCookie,
        ServiceUuid,
        usize,
        oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ),
    DestroyService(ServiceCookie, oneshot::Sender<DestroyServiceResult>),
    SubscribeServicesCreated(mpsc::Sender<ServiceId>, SubscribeMode),
    SubscribeServicesDestroyed(mpsc::Sender<ServiceId>),
    CallFunction(
        ServiceCookie,
        u32,
        Value,
        oneshot::Sender<CallFunctionResult>,
    ),
    FunctionCallReply(u32, CallFunctionResult),
    SubscribeEvent(SubscribeEventRequest),
    UnsubscribeEvent(UnsubscribeEventRequest),
    EmitEvent(EmitEventRequest),
}

#[derive(Debug)]
pub(crate) struct SubscribeEventRequest {
    pub events_id: EventsId,
    pub service_cookie: ServiceCookie,
    pub id: u32,
    pub sender: mpsc::Sender<EventsRequest>,
    pub reply: oneshot::Sender<SubscribeEventResult>,
}

#[derive(Debug)]
pub(crate) struct UnsubscribeEventRequest {
    pub events_id: EventsId,
    pub service_cookie: ServiceCookie,
    pub id: u32,
}

#[derive(Debug)]
pub(crate) struct EmitEventRequest {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub args: Value,
}
