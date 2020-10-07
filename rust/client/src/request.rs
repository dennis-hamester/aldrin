use super::{
    EventsId, EventsRequest, FunctionCallReceiver, ObjectCookie, ObjectEvent, ObjectUuid,
    ServiceCookie, ServiceEvent, ServiceUuid, SubscribeMode,
};
use aldrin_proto::{
    CallFunctionResult, CreateObjectResult, DestroyObjectResult, DestroyServiceResult,
    QueryServiceVersionResult, SubscribeEventResult, Value,
};
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(crate) enum Request {
    Shutdown,
    CreateObject(CreateObjectRequest),
    DestroyObject(DestroyObjectRequest),
    SubscribeObjects(SubscribeObjectsRequest),
    CreateService(CreateServiceRequest),
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
    QueryObject(QueryObjectRequest),
    QueryServiceVersion(ServiceCookie, oneshot::Sender<QueryServiceVersionResult>),
}

#[derive(Debug)]
pub(crate) struct CreateObjectRequest {
    pub uuid: ObjectUuid,
    pub reply: oneshot::Sender<CreateObjectResult>,
}

#[derive(Debug)]
pub(crate) struct DestroyObjectRequest {
    pub cookie: ObjectCookie,
    pub reply: oneshot::Sender<DestroyObjectResult>,
}

#[derive(Debug)]
pub(crate) struct SubscribeObjectsRequest {
    pub mode: SubscribeMode,
    pub sender: mpsc::UnboundedSender<ObjectEvent>,
}

#[derive(Debug)]
pub(crate) enum CreateServiceRequestResult {
    Ok {
        cookie: ServiceCookie,
        function_calls: FunctionCallReceiver,
    },
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug)]
pub(crate) struct CreateServiceRequest {
    pub object_cookie: ObjectCookie,
    pub service_uuid: ServiceUuid,
    pub version: u32,
    pub reply: oneshot::Sender<CreateServiceRequestResult>,
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

pub(crate) type QueryObjectRequestReply = Option<(
    ObjectCookie,
    Option<mpsc::UnboundedReceiver<(ServiceUuid, ServiceCookie)>>,
)>;

#[derive(Debug)]
pub(crate) struct QueryObjectRequest {
    pub object_uuid: ObjectUuid,
    pub reply: oneshot::Sender<QueryObjectRequestReply>,
    pub with_services: bool,
}
