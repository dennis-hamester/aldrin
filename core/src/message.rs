mod abort_function_call;
mod add_bus_listener_filter;
mod add_channel_capacity;
mod bus_listener_current_finished;
mod call_function;
mod call_function_reply;
mod channel_end_claimed;
mod channel_end_closed;
mod claim_channel_end;
mod claim_channel_end_reply;
mod clear_bus_listener_filters;
mod close_channel_end;
mod close_channel_end_reply;
mod connect;
mod connect2;
mod connect_reply;
mod connect_reply2;
mod create_bus_listener;
mod create_bus_listener_reply;
mod create_channel;
mod create_channel_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service2;
mod create_service_reply;
mod destroy_bus_listener;
mod destroy_bus_listener_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod emit_bus_event;
mod emit_event;
mod item_received;
mod packetizer;
mod query_introspection;
mod query_introspection_reply;
mod query_service_info;
mod query_service_version;
mod query_service_version_reply;
mod register_introspection;
mod remove_bus_listener_filter;
mod send_item;
mod service_destroyed;
mod shutdown;
mod start_bus_listener;
mod start_bus_listener_reply;
mod stop_bus_listener;
mod stop_bus_listener_reply;
mod subscribe_event;
mod subscribe_event_reply;
mod sync;
mod sync_reply;
#[cfg(test)]
mod test;
mod unsubscribe_event;

use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub use crate::message_deserializer::MessageDeserializeError;
pub use crate::message_serializer::MessageSerializeError;
pub use abort_function_call::AbortFunctionCall;
pub use add_bus_listener_filter::AddBusListenerFilter;
pub use add_channel_capacity::AddChannelCapacity;
pub use bus_listener_current_finished::BusListenerCurrentFinished;
pub use call_function::CallFunction;
pub use call_function_reply::{CallFunctionReply, CallFunctionResult};
pub use channel_end_claimed::ChannelEndClaimed;
pub use channel_end_closed::ChannelEndClosed;
pub use claim_channel_end::ClaimChannelEnd;
pub use claim_channel_end_reply::{ClaimChannelEndReply, ClaimChannelEndResult};
pub use clear_bus_listener_filters::ClearBusListenerFilters;
pub use close_channel_end::CloseChannelEnd;
pub use close_channel_end_reply::{CloseChannelEndReply, CloseChannelEndResult};
pub use connect::Connect;
pub use connect2::{Connect2, ConnectData};
pub use connect_reply::ConnectReply;
pub use connect_reply2::{ConnectReply2, ConnectReplyData, ConnectResult};
pub use create_bus_listener::CreateBusListener;
pub use create_bus_listener_reply::CreateBusListenerReply;
pub use create_channel::CreateChannel;
pub use create_channel_reply::CreateChannelReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub use create_service::CreateService;
pub use create_service2::CreateService2;
pub use create_service_reply::{CreateServiceReply, CreateServiceResult};
pub use destroy_bus_listener::DestroyBusListener;
pub use destroy_bus_listener_reply::{DestroyBusListenerReply, DestroyBusListenerResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use destroy_service::DestroyService;
pub use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub use emit_bus_event::EmitBusEvent;
pub use emit_event::EmitEvent;
pub use item_received::ItemReceived;
pub use packetizer::Packetizer;
pub use query_introspection::QueryIntrospection;
pub use query_introspection_reply::{QueryIntrospectionReply, QueryIntrospectionResult};
pub use query_service_info::QueryServiceInfo;
pub use query_service_version::QueryServiceVersion;
pub use query_service_version_reply::{QueryServiceVersionReply, QueryServiceVersionResult};
pub use register_introspection::RegisterIntrospection;
pub use remove_bus_listener_filter::RemoveBusListenerFilter;
pub use send_item::SendItem;
pub use service_destroyed::ServiceDestroyed;
pub use shutdown::Shutdown;
pub use start_bus_listener::StartBusListener;
pub use start_bus_listener_reply::{StartBusListenerReply, StartBusListenerResult};
pub use stop_bus_listener::StopBusListener;
pub use stop_bus_listener_reply::{StopBusListenerReply, StopBusListenerResult};
pub use subscribe_event::SubscribeEvent;
pub use subscribe_event_reply::{SubscribeEventReply, SubscribeEventResult};
pub use sync::Sync;
pub use sync_reply::SyncReply;
pub use unsubscribe_event::UnsubscribeEvent;

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageKind {
    Connect = 0,
    ConnectReply = 1,
    Shutdown = 2,
    CreateObject = 3,
    CreateObjectReply = 4,
    DestroyObject = 5,
    DestroyObjectReply = 6,
    CreateService = 7,
    CreateServiceReply = 8,
    DestroyService = 9,
    DestroyServiceReply = 10,
    CallFunction = 11,
    CallFunctionReply = 12,
    SubscribeEvent = 13,
    SubscribeEventReply = 14,
    UnsubscribeEvent = 15,
    EmitEvent = 16,
    QueryServiceVersion = 17,
    QueryServiceVersionReply = 18,
    CreateChannel = 19,
    CreateChannelReply = 20,
    CloseChannelEnd = 21,
    CloseChannelEndReply = 22,
    ChannelEndClosed = 23,
    ClaimChannelEnd = 24,
    ClaimChannelEndReply = 25,
    ChannelEndClaimed = 26,
    SendItem = 27,
    ItemReceived = 28,
    AddChannelCapacity = 29,
    Sync = 30,
    SyncReply = 31,
    ServiceDestroyed = 32,
    CreateBusListener = 33,
    CreateBusListenerReply = 34,
    DestroyBusListener = 35,
    DestroyBusListenerReply = 36,
    AddBusListenerFilter = 37,
    RemoveBusListenerFilter = 38,
    ClearBusListenerFilters = 39,
    StartBusListener = 40,
    StartBusListenerReply = 41,
    StopBusListener = 42,
    StopBusListenerReply = 43,
    EmitBusEvent = 44,
    BusListenerCurrentFinished = 45,
    Connect2 = 46,
    ConnectReply2 = 47,
    AbortFunctionCall = 48,
    RegisterIntrospection = 49,
    QueryIntrospection = 50,
    QueryIntrospectionReply = 51,
    CreateService2 = 52,
    QueryServiceInfo = 53,
}

impl MessageKind {
    pub fn has_value(self) -> bool {
        match self {
            Self::Connect
            | Self::ConnectReply
            | Self::CallFunction
            | Self::CallFunctionReply
            | Self::EmitEvent
            | Self::SendItem
            | Self::ItemReceived
            | Self::Connect2
            | Self::ConnectReply2
            | Self::RegisterIntrospection
            | Self::QueryIntrospectionReply
            | Self::CreateService2 => true,

            Self::Shutdown
            | Self::CreateObject
            | Self::CreateObjectReply
            | Self::DestroyObject
            | Self::DestroyObjectReply
            | Self::CreateService
            | Self::CreateServiceReply
            | Self::DestroyService
            | Self::DestroyServiceReply
            | Self::SubscribeEvent
            | Self::SubscribeEventReply
            | Self::UnsubscribeEvent
            | Self::QueryServiceVersion
            | Self::QueryServiceVersionReply
            | Self::CreateChannel
            | Self::CreateChannelReply
            | Self::CloseChannelEnd
            | Self::CloseChannelEndReply
            | Self::ChannelEndClosed
            | Self::ClaimChannelEnd
            | Self::ClaimChannelEndReply
            | Self::ChannelEndClaimed
            | Self::AddChannelCapacity
            | Self::Sync
            | Self::SyncReply
            | Self::ServiceDestroyed
            | Self::CreateBusListener
            | Self::CreateBusListenerReply
            | Self::DestroyBusListener
            | Self::DestroyBusListenerReply
            | Self::AddBusListenerFilter
            | Self::RemoveBusListenerFilter
            | Self::ClearBusListenerFilters
            | Self::StartBusListener
            | Self::StartBusListenerReply
            | Self::StopBusListener
            | Self::StopBusListenerReply
            | Self::EmitBusEvent
            | Self::BusListenerCurrentFinished
            | Self::AbortFunctionCall
            | Self::QueryIntrospection
            | Self::QueryServiceInfo => false,
        }
    }
}

mod message_ops {
    pub trait Sealed {}
}

pub trait MessageOps: Sized + message_ops::Sealed {
    fn kind(&self) -> MessageKind;
    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError>;
    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError>;
    fn value(&self) -> Option<&SerializedValueSlice>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown(Shutdown),
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
    EmitEvent(EmitEvent),
    QueryServiceVersion(QueryServiceVersion),
    QueryServiceVersionReply(QueryServiceVersionReply),
    CreateChannel(CreateChannel),
    CreateChannelReply(CreateChannelReply),
    CloseChannelEnd(CloseChannelEnd),
    CloseChannelEndReply(CloseChannelEndReply),
    ChannelEndClosed(ChannelEndClosed),
    ClaimChannelEnd(ClaimChannelEnd),
    ClaimChannelEndReply(ClaimChannelEndReply),
    ChannelEndClaimed(ChannelEndClaimed),
    SendItem(SendItem),
    ItemReceived(ItemReceived),
    AddChannelCapacity(AddChannelCapacity),
    Sync(Sync),
    SyncReply(SyncReply),
    ServiceDestroyed(ServiceDestroyed),
    CreateBusListener(CreateBusListener),
    CreateBusListenerReply(CreateBusListenerReply),
    DestroyBusListener(DestroyBusListener),
    DestroyBusListenerReply(DestroyBusListenerReply),
    AddBusListenerFilter(AddBusListenerFilter),
    RemoveBusListenerFilter(RemoveBusListenerFilter),
    ClearBusListenerFilters(ClearBusListenerFilters),
    StartBusListener(StartBusListener),
    StartBusListenerReply(StartBusListenerReply),
    StopBusListener(StopBusListener),
    StopBusListenerReply(StopBusListenerReply),
    EmitBusEvent(EmitBusEvent),
    BusListenerCurrentFinished(BusListenerCurrentFinished),
    Connect2(Connect2),
    ConnectReply2(ConnectReply2),
    AbortFunctionCall(AbortFunctionCall),
    RegisterIntrospection(RegisterIntrospection),
    QueryIntrospection(QueryIntrospection),
    QueryIntrospectionReply(QueryIntrospectionReply),
    CreateService2(CreateService2),
    QueryServiceInfo(QueryServiceInfo),
}

impl MessageOps for Message {
    fn kind(&self) -> MessageKind {
        match self {
            Self::Connect(_) => MessageKind::Connect,
            Self::ConnectReply(_) => MessageKind::ConnectReply,
            Self::Shutdown(_) => MessageKind::Shutdown,
            Self::CreateObject(_) => MessageKind::CreateObject,
            Self::CreateObjectReply(_) => MessageKind::CreateObjectReply,
            Self::DestroyObject(_) => MessageKind::DestroyObject,
            Self::DestroyObjectReply(_) => MessageKind::DestroyObjectReply,
            Self::CreateService(_) => MessageKind::CreateService,
            Self::CreateServiceReply(_) => MessageKind::CreateServiceReply,
            Self::DestroyService(_) => MessageKind::DestroyService,
            Self::DestroyServiceReply(_) => MessageKind::DestroyServiceReply,
            Self::CallFunction(_) => MessageKind::CallFunction,
            Self::CallFunctionReply(_) => MessageKind::CallFunctionReply,
            Self::SubscribeEvent(_) => MessageKind::SubscribeEvent,
            Self::SubscribeEventReply(_) => MessageKind::SubscribeEventReply,
            Self::UnsubscribeEvent(_) => MessageKind::UnsubscribeEvent,
            Self::EmitEvent(_) => MessageKind::EmitEvent,
            Self::QueryServiceVersion(_) => MessageKind::QueryServiceVersion,
            Self::QueryServiceVersionReply(_) => MessageKind::QueryServiceVersionReply,
            Self::CreateChannel(_) => MessageKind::CreateChannel,
            Self::CreateChannelReply(_) => MessageKind::CreateChannelReply,
            Self::CloseChannelEnd(_) => MessageKind::CloseChannelEnd,
            Self::CloseChannelEndReply(_) => MessageKind::CloseChannelEndReply,
            Self::ChannelEndClosed(_) => MessageKind::ChannelEndClosed,
            Self::ClaimChannelEnd(_) => MessageKind::ClaimChannelEnd,
            Self::ClaimChannelEndReply(_) => MessageKind::ClaimChannelEndReply,
            Self::ChannelEndClaimed(_) => MessageKind::ChannelEndClaimed,
            Self::SendItem(_) => MessageKind::SendItem,
            Self::ItemReceived(_) => MessageKind::ItemReceived,
            Self::AddChannelCapacity(_) => MessageKind::AddChannelCapacity,
            Self::Sync(_) => MessageKind::Sync,
            Self::SyncReply(_) => MessageKind::SyncReply,
            Self::ServiceDestroyed(_) => MessageKind::ServiceDestroyed,
            Self::CreateBusListener(_) => MessageKind::CreateBusListener,
            Self::CreateBusListenerReply(_) => MessageKind::CreateBusListenerReply,
            Self::DestroyBusListener(_) => MessageKind::DestroyBusListener,
            Self::DestroyBusListenerReply(_) => MessageKind::DestroyBusListenerReply,
            Self::AddBusListenerFilter(_) => MessageKind::AddBusListenerFilter,
            Self::RemoveBusListenerFilter(_) => MessageKind::RemoveBusListenerFilter,
            Self::ClearBusListenerFilters(_) => MessageKind::ClearBusListenerFilters,
            Self::StartBusListener(_) => MessageKind::StartBusListener,
            Self::StartBusListenerReply(_) => MessageKind::StartBusListenerReply,
            Self::StopBusListener(_) => MessageKind::StopBusListener,
            Self::StopBusListenerReply(_) => MessageKind::StopBusListenerReply,
            Self::EmitBusEvent(_) => MessageKind::EmitBusEvent,
            Self::BusListenerCurrentFinished(_) => MessageKind::BusListenerCurrentFinished,
            Self::Connect2(_) => MessageKind::Connect2,
            Self::ConnectReply2(_) => MessageKind::ConnectReply2,
            Self::AbortFunctionCall(_) => MessageKind::AbortFunctionCall,
            Self::RegisterIntrospection(_) => MessageKind::RegisterIntrospection,
            Self::QueryIntrospection(_) => MessageKind::QueryIntrospection,
            Self::QueryIntrospectionReply(_) => MessageKind::QueryIntrospectionReply,
            Self::CreateService2(_) => MessageKind::CreateService2,
            Self::QueryServiceInfo(_) => MessageKind::QueryServiceInfo,
        }
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        match self {
            Self::Connect(msg) => msg.serialize_message(),
            Self::ConnectReply(msg) => msg.serialize_message(),
            Self::Shutdown(msg) => msg.serialize_message(),
            Self::CreateObject(msg) => msg.serialize_message(),
            Self::CreateObjectReply(msg) => msg.serialize_message(),
            Self::DestroyObject(msg) => msg.serialize_message(),
            Self::DestroyObjectReply(msg) => msg.serialize_message(),
            Self::CreateService(msg) => msg.serialize_message(),
            Self::CreateServiceReply(msg) => msg.serialize_message(),
            Self::DestroyService(msg) => msg.serialize_message(),
            Self::DestroyServiceReply(msg) => msg.serialize_message(),
            Self::CallFunction(msg) => msg.serialize_message(),
            Self::CallFunctionReply(msg) => msg.serialize_message(),
            Self::SubscribeEvent(msg) => msg.serialize_message(),
            Self::SubscribeEventReply(msg) => msg.serialize_message(),
            Self::UnsubscribeEvent(msg) => msg.serialize_message(),
            Self::EmitEvent(msg) => msg.serialize_message(),
            Self::QueryServiceVersion(msg) => msg.serialize_message(),
            Self::QueryServiceVersionReply(msg) => msg.serialize_message(),
            Self::CreateChannel(msg) => msg.serialize_message(),
            Self::CreateChannelReply(msg) => msg.serialize_message(),
            Self::CloseChannelEnd(msg) => msg.serialize_message(),
            Self::CloseChannelEndReply(msg) => msg.serialize_message(),
            Self::ChannelEndClosed(msg) => msg.serialize_message(),
            Self::ClaimChannelEnd(msg) => msg.serialize_message(),
            Self::ClaimChannelEndReply(msg) => msg.serialize_message(),
            Self::ChannelEndClaimed(msg) => msg.serialize_message(),
            Self::SendItem(msg) => msg.serialize_message(),
            Self::ItemReceived(msg) => msg.serialize_message(),
            Self::AddChannelCapacity(msg) => msg.serialize_message(),
            Self::Sync(msg) => msg.serialize_message(),
            Self::SyncReply(msg) => msg.serialize_message(),
            Self::ServiceDestroyed(msg) => msg.serialize_message(),
            Self::CreateBusListener(msg) => msg.serialize_message(),
            Self::CreateBusListenerReply(msg) => msg.serialize_message(),
            Self::DestroyBusListener(msg) => msg.serialize_message(),
            Self::DestroyBusListenerReply(msg) => msg.serialize_message(),
            Self::AddBusListenerFilter(msg) => msg.serialize_message(),
            Self::RemoveBusListenerFilter(msg) => msg.serialize_message(),
            Self::ClearBusListenerFilters(msg) => msg.serialize_message(),
            Self::StartBusListener(msg) => msg.serialize_message(),
            Self::StartBusListenerReply(msg) => msg.serialize_message(),
            Self::StopBusListener(msg) => msg.serialize_message(),
            Self::StopBusListenerReply(msg) => msg.serialize_message(),
            Self::EmitBusEvent(msg) => msg.serialize_message(),
            Self::BusListenerCurrentFinished(msg) => msg.serialize_message(),
            Self::Connect2(msg) => msg.serialize_message(),
            Self::ConnectReply2(msg) => msg.serialize_message(),
            Self::AbortFunctionCall(msg) => msg.serialize_message(),
            Self::RegisterIntrospection(msg) => msg.serialize_message(),
            Self::QueryIntrospection(msg) => msg.serialize_message(),
            Self::QueryIntrospectionReply(msg) => msg.serialize_message(),
            Self::CreateService2(msg) => msg.serialize_message(),
            Self::QueryServiceInfo(msg) => msg.serialize_message(),
        }
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        if buf.len() < 5 {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        match buf[4]
            .try_into()
            .map_err(|_| MessageDeserializeError::InvalidSerialization)?
        {
            MessageKind::Connect => Connect::deserialize_message(buf).map(Self::Connect),
            MessageKind::ConnectReply => {
                ConnectReply::deserialize_message(buf).map(Self::ConnectReply)
            }
            MessageKind::Shutdown => Shutdown::deserialize_message(buf).map(Self::Shutdown),
            MessageKind::CreateObject => {
                CreateObject::deserialize_message(buf).map(Self::CreateObject)
            }
            MessageKind::CreateObjectReply => {
                CreateObjectReply::deserialize_message(buf).map(Self::CreateObjectReply)
            }
            MessageKind::DestroyObject => {
                DestroyObject::deserialize_message(buf).map(Self::DestroyObject)
            }
            MessageKind::DestroyObjectReply => {
                DestroyObjectReply::deserialize_message(buf).map(Self::DestroyObjectReply)
            }
            MessageKind::CreateService => {
                CreateService::deserialize_message(buf).map(Self::CreateService)
            }
            MessageKind::CreateServiceReply => {
                CreateServiceReply::deserialize_message(buf).map(Self::CreateServiceReply)
            }
            MessageKind::DestroyService => {
                DestroyService::deserialize_message(buf).map(Self::DestroyService)
            }
            MessageKind::DestroyServiceReply => {
                DestroyServiceReply::deserialize_message(buf).map(Self::DestroyServiceReply)
            }
            MessageKind::CallFunction => {
                CallFunction::deserialize_message(buf).map(Self::CallFunction)
            }
            MessageKind::CallFunctionReply => {
                CallFunctionReply::deserialize_message(buf).map(Self::CallFunctionReply)
            }
            MessageKind::SubscribeEvent => {
                SubscribeEvent::deserialize_message(buf).map(Self::SubscribeEvent)
            }
            MessageKind::SubscribeEventReply => {
                SubscribeEventReply::deserialize_message(buf).map(Self::SubscribeEventReply)
            }
            MessageKind::UnsubscribeEvent => {
                UnsubscribeEvent::deserialize_message(buf).map(Self::UnsubscribeEvent)
            }
            MessageKind::EmitEvent => EmitEvent::deserialize_message(buf).map(Self::EmitEvent),
            MessageKind::QueryServiceVersion => {
                QueryServiceVersion::deserialize_message(buf).map(Self::QueryServiceVersion)
            }
            MessageKind::QueryServiceVersionReply => {
                QueryServiceVersionReply::deserialize_message(buf)
                    .map(Self::QueryServiceVersionReply)
            }
            MessageKind::CreateChannel => {
                CreateChannel::deserialize_message(buf).map(Self::CreateChannel)
            }
            MessageKind::CreateChannelReply => {
                CreateChannelReply::deserialize_message(buf).map(Self::CreateChannelReply)
            }
            MessageKind::CloseChannelEnd => {
                CloseChannelEnd::deserialize_message(buf).map(Self::CloseChannelEnd)
            }
            MessageKind::CloseChannelEndReply => {
                CloseChannelEndReply::deserialize_message(buf).map(Self::CloseChannelEndReply)
            }
            MessageKind::ChannelEndClosed => {
                ChannelEndClosed::deserialize_message(buf).map(Self::ChannelEndClosed)
            }
            MessageKind::ClaimChannelEnd => {
                ClaimChannelEnd::deserialize_message(buf).map(Self::ClaimChannelEnd)
            }
            MessageKind::ClaimChannelEndReply => {
                ClaimChannelEndReply::deserialize_message(buf).map(Self::ClaimChannelEndReply)
            }
            MessageKind::ChannelEndClaimed => {
                ChannelEndClaimed::deserialize_message(buf).map(Self::ChannelEndClaimed)
            }
            MessageKind::SendItem => SendItem::deserialize_message(buf).map(Self::SendItem),
            MessageKind::ItemReceived => {
                ItemReceived::deserialize_message(buf).map(Self::ItemReceived)
            }
            MessageKind::AddChannelCapacity => {
                AddChannelCapacity::deserialize_message(buf).map(Self::AddChannelCapacity)
            }
            MessageKind::Sync => Sync::deserialize_message(buf).map(Self::Sync),
            MessageKind::SyncReply => SyncReply::deserialize_message(buf).map(Self::SyncReply),
            MessageKind::ServiceDestroyed => {
                ServiceDestroyed::deserialize_message(buf).map(Self::ServiceDestroyed)
            }
            MessageKind::CreateBusListener => {
                CreateBusListener::deserialize_message(buf).map(Self::CreateBusListener)
            }
            MessageKind::CreateBusListenerReply => {
                CreateBusListenerReply::deserialize_message(buf).map(Self::CreateBusListenerReply)
            }
            MessageKind::DestroyBusListener => {
                DestroyBusListener::deserialize_message(buf).map(Self::DestroyBusListener)
            }
            MessageKind::DestroyBusListenerReply => {
                DestroyBusListenerReply::deserialize_message(buf).map(Self::DestroyBusListenerReply)
            }
            MessageKind::AddBusListenerFilter => {
                AddBusListenerFilter::deserialize_message(buf).map(Self::AddBusListenerFilter)
            }
            MessageKind::RemoveBusListenerFilter => {
                RemoveBusListenerFilter::deserialize_message(buf).map(Self::RemoveBusListenerFilter)
            }
            MessageKind::ClearBusListenerFilters => {
                ClearBusListenerFilters::deserialize_message(buf).map(Self::ClearBusListenerFilters)
            }
            MessageKind::StartBusListener => {
                StartBusListener::deserialize_message(buf).map(Self::StartBusListener)
            }
            MessageKind::StartBusListenerReply => {
                StartBusListenerReply::deserialize_message(buf).map(Self::StartBusListenerReply)
            }
            MessageKind::StopBusListener => {
                StopBusListener::deserialize_message(buf).map(Self::StopBusListener)
            }
            MessageKind::StopBusListenerReply => {
                StopBusListenerReply::deserialize_message(buf).map(Self::StopBusListenerReply)
            }
            MessageKind::EmitBusEvent => {
                EmitBusEvent::deserialize_message(buf).map(Self::EmitBusEvent)
            }
            MessageKind::BusListenerCurrentFinished => {
                BusListenerCurrentFinished::deserialize_message(buf)
                    .map(Self::BusListenerCurrentFinished)
            }
            MessageKind::Connect2 => Connect2::deserialize_message(buf).map(Self::Connect2),
            MessageKind::ConnectReply2 => {
                ConnectReply2::deserialize_message(buf).map(Self::ConnectReply2)
            }
            MessageKind::AbortFunctionCall => {
                AbortFunctionCall::deserialize_message(buf).map(Self::AbortFunctionCall)
            }
            MessageKind::RegisterIntrospection => {
                RegisterIntrospection::deserialize_message(buf).map(Self::RegisterIntrospection)
            }
            MessageKind::QueryIntrospection => {
                QueryIntrospection::deserialize_message(buf).map(Self::QueryIntrospection)
            }
            MessageKind::QueryIntrospectionReply => {
                QueryIntrospectionReply::deserialize_message(buf).map(Self::QueryIntrospectionReply)
            }
            MessageKind::CreateService2 => {
                CreateService2::deserialize_message(buf).map(Self::CreateService2)
            }
            MessageKind::QueryServiceInfo => {
                QueryServiceInfo::deserialize_message(buf).map(Self::QueryServiceInfo)
            }
        }
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        match self {
            Self::Connect(msg) => msg.value(),
            Self::ConnectReply(msg) => msg.value(),
            Self::Shutdown(msg) => msg.value(),
            Self::CreateObject(msg) => msg.value(),
            Self::CreateObjectReply(msg) => msg.value(),
            Self::DestroyObject(msg) => msg.value(),
            Self::DestroyObjectReply(msg) => msg.value(),
            Self::CreateService(msg) => msg.value(),
            Self::CreateServiceReply(msg) => msg.value(),
            Self::DestroyService(msg) => msg.value(),
            Self::DestroyServiceReply(msg) => msg.value(),
            Self::CallFunction(msg) => msg.value(),
            Self::CallFunctionReply(msg) => msg.value(),
            Self::SubscribeEvent(msg) => msg.value(),
            Self::SubscribeEventReply(msg) => msg.value(),
            Self::UnsubscribeEvent(msg) => msg.value(),
            Self::EmitEvent(msg) => msg.value(),
            Self::QueryServiceVersion(msg) => msg.value(),
            Self::QueryServiceVersionReply(msg) => msg.value(),
            Self::CreateChannel(msg) => msg.value(),
            Self::CreateChannelReply(msg) => msg.value(),
            Self::CloseChannelEnd(msg) => msg.value(),
            Self::CloseChannelEndReply(msg) => msg.value(),
            Self::ChannelEndClosed(msg) => msg.value(),
            Self::ClaimChannelEnd(msg) => msg.value(),
            Self::ClaimChannelEndReply(msg) => msg.value(),
            Self::ChannelEndClaimed(msg) => msg.value(),
            Self::SendItem(msg) => msg.value(),
            Self::ItemReceived(msg) => msg.value(),
            Self::AddChannelCapacity(msg) => msg.value(),
            Self::Sync(msg) => msg.value(),
            Self::SyncReply(msg) => msg.value(),
            Self::ServiceDestroyed(msg) => msg.value(),
            Self::CreateBusListener(msg) => msg.value(),
            Self::CreateBusListenerReply(msg) => msg.value(),
            Self::DestroyBusListener(msg) => msg.value(),
            Self::DestroyBusListenerReply(msg) => msg.value(),
            Self::AddBusListenerFilter(msg) => msg.value(),
            Self::RemoveBusListenerFilter(msg) => msg.value(),
            Self::ClearBusListenerFilters(msg) => msg.value(),
            Self::StartBusListener(msg) => msg.value(),
            Self::StartBusListenerReply(msg) => msg.value(),
            Self::StopBusListener(msg) => msg.value(),
            Self::StopBusListenerReply(msg) => msg.value(),
            Self::EmitBusEvent(msg) => msg.value(),
            Self::BusListenerCurrentFinished(msg) => msg.value(),
            Self::Connect2(msg) => msg.value(),
            Self::ConnectReply2(msg) => msg.value(),
            Self::AbortFunctionCall(msg) => msg.value(),
            Self::RegisterIntrospection(msg) => msg.value(),
            Self::QueryIntrospection(msg) => msg.value(),
            Self::QueryIntrospectionReply(msg) => msg.value(),
            Self::CreateService2(msg) => msg.value(),
            Self::QueryServiceInfo(msg) => msg.value(),
        }
    }
}

impl message_ops::Sealed for Message {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum OptionKind {
    None = 0,
    Some = 1,
}
