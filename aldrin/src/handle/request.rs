use crate::bus_listener::BusListener;
use crate::channel::{
    PendingReceiverInner, PendingSenderInner, ReceiverInner, SenderInner, UnclaimedReceiverInner,
    UnclaimedSenderInner,
};
#[cfg(feature = "introspection")]
use crate::core::introspection::Introspection;
use crate::core::message::{
    AddBusListenerFilter, AddChannelCapacity, CallFunctionResult, ClearBusListenerFilters,
    DestroyBusListenerResult, DestroyObjectResult, RemoveBusListenerFilter, StartBusListenerResult,
    StopBusListenerResult,
};
#[cfg(feature = "introspection")]
use crate::core::TypeId;
use crate::core::{
    BusListenerCookie, BusListenerScope, ChannelCookie, ChannelEnd, ObjectCookie, ObjectId,
    ObjectUuid, ProtocolVersion, SerializedValue, ServiceCookie, ServiceId, ServiceInfo,
    ServiceUuid,
};
use crate::lifetime::LifetimeListener;
use crate::low_level::Service;
use crate::{Error, Object};
use futures_channel::oneshot;
use std::num::NonZeroU32;

#[derive(Debug)]
pub(crate) enum HandleRequest {
    HandleCloned,
    HandleDropped,
    Shutdown,
    CreateObject(CreateObjectRequest),
    DestroyObject(DestroyObjectRequest),
    CreateService(CreateServiceRequest),
    DestroyService(DestroyServiceRequest),
    CallFunction(CallFunctionRequest),
    CallFunctionReply(CallFunctionReplyRequest),
    EmitEvent(EmitEventRequest),
    QueryServiceInfo(QueryServiceInfoRequest),
    CreateClaimedSender(CreateClaimedSenderRequest),
    CreateClaimedReceiver(CreateClaimedReceiverRequest),
    CloseChannelEnd(CloseChannelEndRequest),
    ClaimSender(ClaimSenderRequest),
    ClaimReceiver(ClaimReceiverRequest),
    SendItem(SendItemRequest),
    AddChannelCapacity(AddChannelCapacity),
    SyncClient(SyncClientRequest),
    SyncBroker(SyncBrokerRequest),
    CreateBusListener(CreateBusListenerRequest),
    DestroyBusListener(DestroyBusListenerRequest),
    AddBusListenerFilter(AddBusListenerFilter),
    RemoveBusListenerFilter(RemoveBusListenerFilter),
    ClearBusListenerFilters(ClearBusListenerFilters),
    StartBusListener(StartBusListenerRequest),
    StopBusListener(StopBusListenerRequest),
    CreateLifetimeListener(CreateLifetimeListenerRequest),
    GetProtocolVersion(GetProtocolVersionRequest),
    #[cfg(feature = "introspection")]
    RegisterIntrospection(&'static Introspection),
    #[cfg(feature = "introspection")]
    SubmitIntrospection,
    #[cfg(feature = "introspection")]
    QueryIntrospection(QueryIntrospectionRequest),
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
pub(crate) struct CreateServiceRequest {
    pub object_id: ObjectId,
    pub service_uuid: ServiceUuid,
    pub info: ServiceInfo,
    pub reply: oneshot::Sender<Result<Service, Error>>,
}

#[derive(Debug)]
pub(crate) struct DestroyServiceRequest {
    pub id: ServiceId,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct CallFunctionRequest {
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub value: SerializedValue,
    pub reply: oneshot::Sender<Result<CallFunctionResult, Error>>,
}

#[derive(Debug)]
pub(crate) struct CallFunctionReplyRequest {
    pub serial: u32,
    pub result: CallFunctionResult,
}

#[derive(Debug)]
pub(crate) struct EmitEventRequest {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub value: SerializedValue,
}

#[derive(Debug)]
pub(crate) struct QueryServiceInfoRequest {
    pub cookie: ServiceCookie,
    pub reply: oneshot::Sender<Result<ServiceInfo, Error>>,
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

pub(crate) type CreateBusListenerRequest = oneshot::Sender<BusListener>;

#[derive(Debug)]
pub(crate) struct DestroyBusListenerRequest {
    pub cookie: BusListenerCookie,
    pub reply: oneshot::Sender<DestroyBusListenerResult>,
}

#[derive(Debug)]
pub(crate) struct StartBusListenerRequest {
    pub cookie: BusListenerCookie,
    pub scope: BusListenerScope,
    pub reply: oneshot::Sender<StartBusListenerResult>,
}

#[derive(Debug)]
pub(crate) struct StopBusListenerRequest {
    pub cookie: BusListenerCookie,
    pub reply: oneshot::Sender<StopBusListenerResult>,
}

pub(crate) type CreateLifetimeListenerRequest = oneshot::Sender<LifetimeListener>;

pub(crate) type GetProtocolVersionRequest = oneshot::Sender<ProtocolVersion>;

#[cfg(feature = "introspection")]
#[derive(Debug)]
pub(crate) struct QueryIntrospectionRequest {
    pub type_id: TypeId,
    pub reply: oneshot::Sender<Option<IntrospectionQueryResult>>,
}

#[cfg(feature = "introspection")]
#[derive(Debug)]
pub(crate) enum IntrospectionQueryResult {
    Local(&'static Introspection),
    Serialized(SerializedValue),
}
