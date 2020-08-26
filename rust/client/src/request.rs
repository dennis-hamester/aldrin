use super::{
    EventsId, EventsRequest, FunctionCallReceiver, ObjectCookie, ObjectEvent, ObjectUuid,
    ServiceCookie, ServiceEvent, ServiceUuid, SubscribeMode,
};
use aldrin_proto::{
    CallFunctionResult, CreateObjectResult, CreateServiceResult, DestroyObjectResult,
    DestroyServiceResult, SubscribeEventResult, Value,
};
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(crate) enum Request {
    Shutdown,
    CreateObject(ObjectUuid, oneshot::Sender<CreateObjectResult>),
    DestroyObject(ObjectCookie, oneshot::Sender<DestroyObjectResult>),
    SubscribeObjects(mpsc::UnboundedSender<ObjectEvent>, SubscribeMode),
    CreateService(
        ObjectCookie,
        ServiceUuid,
        oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ),
    DestroyService(ServiceCookie, oneshot::Sender<DestroyServiceResult>),
    SubscribeServices(mpsc::UnboundedSender<ServiceEvent>, SubscribeMode),
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
    pub sender: mpsc::UnboundedSender<EventsRequest>,
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
