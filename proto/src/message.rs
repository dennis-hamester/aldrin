mod add_bus_listener_filter;
mod add_channel_capacity;
mod bus_listener_filter;
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
mod connect_reply;
mod create_bus_listener;
mod create_bus_listener_reply;
mod create_channel;
mod create_channel_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service_reply;
mod destroy_bus_listener;
mod destroy_bus_listener_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod emit_event;
mod item_received;
mod packetizer;
mod query_service_version;
mod query_service_version_reply;
mod remove_bus_listener_filter;
mod send_item;
mod service_destroyed;
mod shutdown;
mod subscribe_event;
mod subscribe_event_reply;
mod sync;
mod sync_reply;
#[cfg(test)]
mod test;
mod unsubscribe_event;

use crate::buf_ext::{BufMutExt, MessageBufExt};
use crate::error::{DeserializeError, SerializeError};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::{Buf, BufMut, BytesMut};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::error::Error;
use std::fmt;
use uuid::Uuid;

pub use add_bus_listener_filter::AddBusListenerFilter;
pub use add_channel_capacity::AddChannelCapacity;
pub use bus_listener_filter::{BusListenerFilter, BusListenerServiceFilter};
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
pub use connect_reply::ConnectReply;
pub use create_bus_listener::CreateBusListener;
pub use create_bus_listener_reply::CreateBusListenerReply;
pub use create_channel::CreateChannel;
pub use create_channel_reply::CreateChannelReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub use create_service::CreateService;
pub use create_service_reply::{CreateServiceReply, CreateServiceResult};
pub use destroy_bus_listener::DestroyBusListener;
pub use destroy_bus_listener_reply::{DestroyBusListenerReply, DestroyBusListenerResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use destroy_service::DestroyService;
pub use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub use emit_event::EmitEvent;
pub use item_received::ItemReceived;
pub use packetizer::Packetizer;
pub use query_service_version::QueryServiceVersion;
pub use query_service_version_reply::{QueryServiceVersionReply, QueryServiceVersionResult};
pub use remove_bus_listener_filter::RemoveBusListenerFilter;
pub use send_item::SendItem;
pub use service_destroyed::ServiceDestroyed;
pub use shutdown::Shutdown;
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
            | Self::ItemReceived => true,

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
            | Self::ClearBusListenerFilters => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageSerializeError {
    Overflow,
    InvalidValue,
}

impl fmt::Display for MessageSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("serialized message overflowed"),
            Self::InvalidValue => f.write_str("invalid value"),
        }
    }
}

impl Error for MessageSerializeError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageDeserializeError {
    InvalidSerialization,
    UnexpectedEoi,
    UnexpectedMessage,
    TrailingData,
}

impl fmt::Display for MessageDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidSerialization => f.write_str("invalid serialization"),
            Self::UnexpectedEoi => f.write_str("unexpected end of input"),
            Self::UnexpectedMessage => f.write_str("unexpected message type"),
            Self::TrailingData => f.write_str("serialization contains trailing data"),
        }
    }
}

impl Error for MessageDeserializeError {}

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
        }
    }
}

impl message_ops::Sealed for Message {}

/// Sending or receiving end of a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum ChannelEnd {
    /// Sending end of a channel.
    Sender = 0,

    /// Receiving end of a channel.
    Receiver = 1,
}

impl ChannelEnd {
    /// Returns the other end of the channel.
    ///
    /// This function maps [`Sender`](Self::Sender) to [`Receiver`](Self::Receiver) and vice versa.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::message::ChannelEnd;
    /// assert_eq!(ChannelEnd::Sender.other(), ChannelEnd::Receiver);
    /// assert_eq!(ChannelEnd::Receiver.other(), ChannelEnd::Sender);
    /// ```
    pub fn other(self) -> Self {
        match self {
            Self::Sender => Self::Receiver,
            Self::Receiver => Self::Sender,
        }
    }
}

impl From<ChannelEndWithCapacity> for ChannelEnd {
    fn from(value: ChannelEndWithCapacity) -> Self {
        match value {
            ChannelEndWithCapacity::Sender => Self::Sender,
            ChannelEndWithCapacity::Receiver(_) => Self::Receiver,
        }
    }
}

impl Serialize for ChannelEnd {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Sender => serializer.serialize_enum(0, &()),
            Self::Receiver => serializer.serialize_enum(1, &()),
        }
    }
}

impl Deserialize for ChannelEnd {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.variant() {
            0 => deserializer.deserialize().map(|()| Self::Sender),
            1 => deserializer.deserialize().map(|()| Self::Receiver),
            _ => Err(DeserializeError::InvalidSerialization),
        }
    }
}

/// Sending or receiving end and capacity of a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum ChannelEndWithCapacity {
    /// Sending end of a channel.
    Sender,

    /// Receiving end of a channel and capacity.
    Receiver(u32),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum OptionKind {
    None = 0,
    Some = 1,
}

struct MessageSerializer {
    buf: BytesMut,
}

impl MessageSerializer {
    fn without_value(kind: MessageKind) -> Self {
        debug_assert!(!kind.has_value());

        let mut buf = BytesMut::zeroed(4);
        buf.put_u8(kind.into());

        Self { buf }
    }

    fn with_value(
        value: SerializedValue,
        kind: MessageKind,
    ) -> Result<Self, MessageSerializeError> {
        debug_assert!(kind.has_value());

        let mut buf = value.into_bytes_mut();

        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        if buf.len() < 10 {
            return Err(MessageSerializeError::InvalidValue);
        }

        let value_len = buf.len() - 9;
        if value_len > u32::MAX as usize {
            return Err(MessageSerializeError::Overflow);
        }

        buf[4] = kind.into();
        buf[5..9].copy_from_slice(&(value_len as u32).to_le_bytes());

        Ok(Self { buf })
    }

    fn with_none_value(kind: MessageKind) -> Self {
        Self::with_value(SerializedValue::serialize(&()).unwrap(), kind).unwrap()
    }

    fn put_discriminant_u8(&mut self, discriminant: impl Into<u8>) {
        self.buf.put_discriminant_u8(discriminant);
    }

    fn put_varint_u32_le(&mut self, n: u32) {
        self.buf.put_varint_u32_le(n);
    }

    fn put_uuid(&mut self, uuid: Uuid) {
        self.buf.put_slice(uuid.as_ref());
    }

    fn finish(mut self) -> Result<BytesMut, MessageSerializeError> {
        let len = self.buf.len();
        if len <= u32::MAX as usize {
            self.buf[..4].copy_from_slice(&(len as u32).to_le_bytes());
            Ok(self.buf)
        } else {
            Err(MessageSerializeError::Overflow)
        }
    }
}

struct MessageWithoutValueDeserializer {
    buf: BytesMut,
}

impl MessageWithoutValueDeserializer {
    fn new(mut buf: BytesMut, kind: MessageKind) -> Result<Self, MessageDeserializeError> {
        let buf_len = buf.len();

        // 4 bytes message length + 1 byte message kind.
        if buf_len < 5 {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let len = buf.get_u32_le() as usize;
        if buf_len != len {
            return Err(MessageDeserializeError::InvalidSerialization);
        }

        buf.ensure_discriminant_u8(kind)?;

        Ok(Self { buf })
    }

    fn try_get_discriminant_u8<T: TryFrom<u8>>(&mut self) -> Result<T, MessageDeserializeError> {
        self.buf.try_get_discriminant_u8()
    }

    fn try_get_varint_u32_le(&mut self) -> Result<u32, MessageDeserializeError> {
        self.buf.try_get_varint_u32_le()
    }

    fn try_get_uuid(&mut self) -> Result<Uuid, MessageDeserializeError> {
        let mut bytes = uuid::Bytes::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    fn finish(self) -> Result<(), MessageDeserializeError> {
        if self.buf.is_empty() {
            Ok(())
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }
}

struct MessageWithValueDeserializer {
    header_and_value: BytesMut,
    msg: BytesMut,
}

impl MessageWithValueDeserializer {
    fn new(mut buf: BytesMut, kind: MessageKind) -> Result<Self, MessageDeserializeError> {
        debug_assert!(kind.has_value());

        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        if buf.len() < 10 {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let msg_len = (&buf[..4]).get_u32_le() as usize;
        if buf.len() != msg_len {
            return Err(MessageDeserializeError::InvalidSerialization);
        }

        if buf[4] != kind.into() {
            return Err(MessageDeserializeError::UnexpectedMessage);
        }

        let value_len = (&buf[5..9]).get_u32_le() as usize;
        let max_value_len = buf.len() - 9;

        if value_len < 1 {
            return Err(MessageDeserializeError::InvalidSerialization);
        } else if value_len > max_value_len {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let msg = buf.split_off(9 + value_len);
        Ok(Self {
            header_and_value: buf,
            msg,
        })
    }

    fn try_get_discriminant_u8<T: TryFrom<u8>>(&mut self) -> Result<T, MessageDeserializeError> {
        self.msg.try_get_discriminant_u8()
    }

    fn try_get_varint_u32_le(&mut self) -> Result<u32, MessageDeserializeError> {
        self.msg.try_get_varint_u32_le()
    }

    fn try_get_uuid(&mut self) -> Result<Uuid, MessageDeserializeError> {
        let mut bytes = uuid::Bytes::default();
        self.msg.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    fn finish(mut self) -> Result<SerializedValue, MessageDeserializeError> {
        if self.msg.is_empty() {
            self.header_and_value.unsplit(self.msg);
            self.header_and_value[0..9].fill(0);
            Ok(SerializedValue::from_bytes_mut(self.header_and_value))
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }

    fn finish_discard_value(self) -> Result<(), MessageDeserializeError> {
        if self.msg.is_empty() {
            Ok(())
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }
}
