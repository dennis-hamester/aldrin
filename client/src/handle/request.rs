use crate::channel::{
    PendingReceiverInner, PendingSenderInner, ReceiverInner, SenderInner, UnclaimedReceiverInner,
    UnclaimedSenderInner,
};
use crate::events::{EventsId, EventsRequest};
use crate::{Error, Object, ObjectEvent, Service, ServiceEvent, SubscribeMode};
use aldrin_proto::message::{
    AddChannelCapacity, CallFunctionResult, ChannelEnd, DestroyObjectResult,
    QueryServiceVersionResult, SubscribeEventResult,
};
use aldrin_proto::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, SerializedValue, ServiceCookie, ServiceId,
    ServiceUuid,
};
use futures_channel::{mpsc, oneshot};
use std::num::NonZeroU32;

#[derive(Debug)]
pub(crate) enum HandleRequest {
    HandleCloned,
    HandleDropped,
    Shutdown,
    CreateObject(CreateObjectRequest),
    DestroyObject(DestroyObjectRequest),
    SubscribeObjects(SubscribeObjectsRequest),
    CreateService(CreateServiceRequest),
    DestroyService(DestroyServiceRequest),
    SubscribeServices(SubscribeServicesRequest),
    CallFunction(CallFunctionRequest),
    CallFunctionReply(CallFunctionReplyRequest),
    SubscribeEvent(SubscribeEventRequest),
    UnsubscribeEvent(UnsubscribeEventRequest),
    EmitEvent(EmitEventRequest),
    QueryObject(QueryObjectRequest),
    QueryServiceVersion(QueryServiceVersionRequest),
    CreateClaimedSender(CreateClaimedSenderRequest),
    CreateClaimedReceiver(CreateClaimedReceiverRequest),
    CloseChannelEnd(CloseChannelEndRequest),
    ClaimSender(ClaimSenderRequest),
    ClaimReceiver(ClaimReceiverRequest),
    SendItem(SendItemRequest),
    AddChannelCapacity(AddChannelCapacity),
    SyncClient(SyncClientRequest),
    SyncBroker(SyncBrokerRequest),
}

#[derive(Debug)]
pub(crate) struct CreateObjectRequest {
    pub uuid: ObjectUuid,
    pub reply: oneshot::Sender<Result<Object, Error>>,
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
pub(crate) struct CreateServiceRequest {
    pub object_id: ObjectId,
    pub service_uuid: ServiceUuid,
    pub version: u32,
    pub reply: oneshot::Sender<Result<Service, Error>>,
}

#[derive(Debug)]
pub(crate) struct DestroyServiceRequest {
    pub id: ServiceId,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct SubscribeServicesRequest {
    pub mode: SubscribeMode,
    pub sender: mpsc::UnboundedSender<ServiceEvent>,
}

#[derive(Debug)]
pub(crate) struct CallFunctionRequest {
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub value: SerializedValue,
    pub reply: oneshot::Sender<CallFunctionResult>,
}

#[derive(Debug)]
pub(crate) struct CallFunctionReplyRequest {
    pub serial: u32,
    pub result: CallFunctionResult,
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
    pub value: SerializedValue,
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

#[derive(Debug)]
pub(crate) struct QueryServiceVersionRequest {
    pub cookie: ServiceCookie,
    pub reply: oneshot::Sender<QueryServiceVersionResult>,
}

pub(crate) type CreateClaimedSenderRequest =
    oneshot::Sender<(PendingSenderInner, UnclaimedReceiverInner)>;

#[derive(Debug)]
pub(crate) struct CreateClaimedReceiverRequest {
    pub capacity: NonZeroU32,
    pub reply: oneshot::Sender<(UnclaimedSenderInner, PendingReceiverInner)>,
}

#[derive(Debug)]
pub(crate) struct CloseChannelEndRequest {
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
    pub claimed: bool,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct ClaimSenderRequest {
    pub cookie: ChannelCookie,
    pub reply: oneshot::Sender<Result<SenderInner, Error>>,
}

#[derive(Debug)]
pub(crate) struct ClaimReceiverRequest {
    pub cookie: ChannelCookie,
    pub capacity: NonZeroU32,
    pub reply: oneshot::Sender<Result<ReceiverInner, Error>>,
}

#[derive(Debug)]
pub(crate) struct SendItemRequest {
    pub cookie: ChannelCookie,
    pub value: SerializedValue,
}

pub(crate) type SyncClientRequest = oneshot::Sender<()>;

pub(crate) type SyncBrokerRequest = oneshot::Sender<()>;
