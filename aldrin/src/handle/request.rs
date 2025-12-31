use crate::bus_listener::BusListener;
use crate::lifetime::LifetimeListener;
use crate::low_level::{
    PendingReceiver, PendingSender, Proxy, ProxyId, Service, ServiceInfo, UnclaimedReceiver,
    UnclaimedSender,
};
use crate::{Error, Object};
#[cfg(feature = "introspection")]
use aldrin_core::TypeId;
#[cfg(feature = "introspection")]
use aldrin_core::introspection::DynIntrospectable;
use aldrin_core::message::{
    AddBusListenerFilter, AddChannelCapacity, CallFunctionResult, ClearBusListenerFilters,
    DestroyBusListenerResult, DestroyObjectResult, RemoveBusListenerFilter, StartBusListenerResult,
    StopBusListenerResult,
};
use aldrin_core::{
    BusListenerCookie, BusListenerScope, ChannelCookie, ChannelEnd, ObjectCookie, ObjectId,
    ObjectUuid, ProtocolVersion, SerializedValue, ServiceCookie, ServiceId, ServiceUuid,
};
use futures_channel::{mpsc, oneshot};
use std::num::NonZeroU32;
use std::time::Instant;

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
    CreateProxy(CreateProxyRequest),
    DestroyProxy(ProxyId),
    SubscribeEvent(SubscribeEventRequest),
    UnsubscribeEvent(UnsubscribeEventRequest),
    SubscribeAllEvents(SubscribeAllEventsRequest),
    UnsubscribeAllEvents(UnsubscribeAllEventsRequest),
    #[cfg(feature = "introspection")]
    RegisterIntrospection(DynIntrospectable),
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
    pub version: Option<u32>,
    pub value: SerializedValue,
    pub reply: oneshot::Sender<Result<(CallFunctionResult, Instant), Error>>,
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

pub(crate) type CreateClaimedSenderRequest = oneshot::Sender<(PendingSender, UnclaimedReceiver)>;

#[derive(Debug)]
pub(crate) struct CreateClaimedReceiverRequest {
    pub capacity: NonZeroU32,
    pub reply: oneshot::Sender<(UnclaimedSender, PendingReceiver)>,
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
    pub reply: oneshot::Sender<Result<(mpsc::UnboundedReceiver<u32>, u32), Error>>,
}

#[derive(Debug)]
pub(crate) struct ClaimReceiverRequest {
    pub cookie: ChannelCookie,
    pub capacity: NonZeroU32,
    pub reply:
        oneshot::Sender<Result<(mpsc::UnboundedReceiver<SerializedValue>, NonZeroU32), Error>>,
}

#[derive(Debug)]
pub(crate) struct SendItemRequest {
    pub cookie: ChannelCookie,
    pub value: SerializedValue,
}

pub(crate) type SyncClientRequest = oneshot::Sender<Instant>;

pub(crate) type SyncBrokerRequest = oneshot::Sender<Instant>;

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

#[derive(Debug)]
pub(crate) struct CreateProxyRequest {
    pub service: ServiceId,
    pub reply: oneshot::Sender<Result<Proxy, Error>>,
}

#[derive(Debug)]
pub(crate) struct SubscribeEventRequest {
    pub proxy: ProxyId,
    pub event: u32,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct UnsubscribeEventRequest {
    pub proxy: ProxyId,
    pub event: u32,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct SubscribeAllEventsRequest {
    pub proxy: ProxyId,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub(crate) struct UnsubscribeAllEventsRequest {
    pub proxy: ProxyId,
    pub reply: oneshot::Sender<Result<(), Error>>,
}

#[cfg(feature = "introspection")]
#[derive(Debug)]
pub(crate) struct QueryIntrospectionRequest {
    pub type_id: TypeId,
    pub reply: oneshot::Sender<Option<SerializedValue>>,
}
