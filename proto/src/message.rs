mod call_function;
mod call_function_reply;
mod channel_end_claimed;
mod channel_end_closed;
mod claim_channel_end;
mod claim_channel_end_reply;
mod close_channel_end;
mod close_channel_end_reply;
mod connect;
mod connect_reply;
mod create_channel;
mod create_channel_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod emit_event;
mod item_received;
mod object_created_event;
mod object_destroyed_event;
mod packetizer;
mod query_object;
mod query_object_reply;
mod query_service_version;
mod query_service_version_reply;
mod send_item;
mod service_created_event;
mod service_destroyed_event;
mod shutdown;
mod subscribe_event;
mod subscribe_event_reply;
mod subscribe_objects;
mod subscribe_objects_reply;
mod subscribe_services;
mod subscribe_services_reply;
mod sync;
mod sync_reply;
#[cfg(test)]
mod test;
mod unsubscribe_event;
mod unsubscribe_objects;
mod unsubscribe_services;

use crate::buf_ext::{BufMutExt, MessageBufExt};
use crate::error::{DeserializeError, SerializeError};
use crate::serialized_value::SerializedValue;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::{Buf, BufMut, BytesMut};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::error::Error;
use std::fmt;
use uuid::Uuid;

pub use call_function::CallFunction;
pub use call_function_reply::{CallFunctionReply, CallFunctionReplyKind, CallFunctionResult};
pub use channel_end_claimed::ChannelEndClaimed;
pub use channel_end_closed::ChannelEndClosed;
pub use claim_channel_end::ClaimChannelEnd;
pub use claim_channel_end_reply::{ClaimChannelEndReply, ClaimChannelEndResult};
pub use close_channel_end::CloseChannelEnd;
pub use close_channel_end_reply::{CloseChannelEndReply, CloseChannelEndResult};
pub use connect::Connect;
pub use connect_reply::{ConnectReply, ConnectReplyKind};
pub use create_channel::CreateChannel;
pub use create_channel_reply::CreateChannelReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectReplyKind, CreateObjectResult};
pub use create_service::CreateService;
pub use create_service_reply::{CreateServiceReply, CreateServiceReplyKind, CreateServiceResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use destroy_service::DestroyService;
pub use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub use emit_event::EmitEvent;
pub use item_received::ItemReceived;
pub use object_created_event::ObjectCreatedEvent;
pub use object_destroyed_event::ObjectDestroyedEvent;
pub use packetizer::Packetizer;
pub use query_object::QueryObject;
pub use query_object_reply::{QueryObjectReply, QueryObjectReplyKind, QueryObjectResult};
pub use query_service_version::QueryServiceVersion;
pub use query_service_version_reply::{
    QueryServiceVersionReply, QueryServiceVersionReplyKind, QueryServiceVersionResult,
};
pub use send_item::SendItem;
pub use service_created_event::ServiceCreatedEvent;
pub use service_destroyed_event::ServiceDestroyedEvent;
pub use shutdown::Shutdown;
pub use subscribe_event::SubscribeEvent;
pub use subscribe_event_reply::{SubscribeEventReply, SubscribeEventResult};
pub use subscribe_objects::SubscribeObjects;
pub use subscribe_objects_reply::SubscribeObjectsReply;
pub use subscribe_services::SubscribeServices;
pub use subscribe_services_reply::SubscribeServicesReply;
pub use sync::Sync;
pub use sync_reply::SyncReply;
pub use unsubscribe_event::UnsubscribeEvent;
pub use unsubscribe_objects::UnsubscribeObjects;
pub use unsubscribe_services::UnsubscribeServices;

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
    SubscribeObjects = 7,
    SubscribeObjectsReply = 8,
    UnsubscribeObjects = 9,
    ObjectCreatedEvent = 10,
    ObjectDestroyedEvent = 11,
    CreateService = 12,
    CreateServiceReply = 13,
    DestroyService = 14,
    DestroyServiceReply = 15,
    SubscribeServices = 16,
    SubscribeServicesReply = 17,
    UnsubscribeServices = 18,
    ServiceCreatedEvent = 19,
    ServiceDestroyedEvent = 20,
    CallFunction = 21,
    CallFunctionReply = 22,
    SubscribeEvent = 23,
    SubscribeEventReply = 24,
    UnsubscribeEvent = 25,
    EmitEvent = 26,
    QueryObject = 27,
    QueryObjectReply = 28,
    QueryServiceVersion = 29,
    QueryServiceVersionReply = 30,
    CreateChannel = 31,
    CreateChannelReply = 32,
    CloseChannelEnd = 33,
    CloseChannelEndReply = 34,
    ChannelEndClosed = 35,
    ClaimChannelEnd = 36,
    ClaimChannelEndReply = 37,
    ChannelEndClaimed = 38,
    SendItem = 39,
    ItemReceived = 40,
    Sync = 42,
    SyncReply = 43,
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
            | Self::SubscribeObjects
            | Self::SubscribeObjectsReply
            | Self::UnsubscribeObjects
            | Self::ObjectCreatedEvent
            | Self::ObjectDestroyedEvent
            | Self::CreateService
            | Self::CreateServiceReply
            | Self::DestroyService
            | Self::DestroyServiceReply
            | Self::SubscribeServices
            | Self::SubscribeServicesReply
            | Self::UnsubscribeServices
            | Self::ServiceCreatedEvent
            | Self::ServiceDestroyedEvent
            | Self::SubscribeEvent
            | Self::SubscribeEventReply
            | Self::UnsubscribeEvent
            | Self::QueryObject
            | Self::QueryObjectReply
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
            | Self::Sync
            | Self::SyncReply => false,
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
    fn value(&self) -> Option<&SerializedValue>;
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
    SubscribeObjects(SubscribeObjects),
    SubscribeObjectsReply(SubscribeObjectsReply),
    UnsubscribeObjects(UnsubscribeObjects),
    ObjectCreatedEvent(ObjectCreatedEvent),
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServices(SubscribeServices),
    SubscribeServicesReply(SubscribeServicesReply),
    UnsubscribeServices(UnsubscribeServices),
    ServiceCreatedEvent(ServiceCreatedEvent),
    ServiceDestroyedEvent(ServiceDestroyedEvent),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
    EmitEvent(EmitEvent),
    QueryObject(QueryObject),
    QueryObjectReply(QueryObjectReply),
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
    Sync(Sync),
    SyncReply(SyncReply),
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
            Self::SubscribeObjects(_) => MessageKind::SubscribeObjects,
            Self::SubscribeObjectsReply(_) => MessageKind::SubscribeObjectsReply,
            Self::UnsubscribeObjects(_) => MessageKind::UnsubscribeObjects,
            Self::ObjectCreatedEvent(_) => MessageKind::ObjectCreatedEvent,
            Self::ObjectDestroyedEvent(_) => MessageKind::ObjectDestroyedEvent,
            Self::CreateService(_) => MessageKind::CreateService,
            Self::CreateServiceReply(_) => MessageKind::CreateServiceReply,
            Self::DestroyService(_) => MessageKind::DestroyService,
            Self::DestroyServiceReply(_) => MessageKind::DestroyServiceReply,
            Self::SubscribeServices(_) => MessageKind::SubscribeServices,
            Self::SubscribeServicesReply(_) => MessageKind::SubscribeServicesReply,
            Self::UnsubscribeServices(_) => MessageKind::UnsubscribeServices,
            Self::ServiceCreatedEvent(_) => MessageKind::ServiceCreatedEvent,
            Self::ServiceDestroyedEvent(_) => MessageKind::ServiceDestroyedEvent,
            Self::CallFunction(_) => MessageKind::CallFunction,
            Self::CallFunctionReply(_) => MessageKind::CallFunctionReply,
            Self::SubscribeEvent(_) => MessageKind::SubscribeEvent,
            Self::SubscribeEventReply(_) => MessageKind::SubscribeEventReply,
            Self::UnsubscribeEvent(_) => MessageKind::UnsubscribeEvent,
            Self::EmitEvent(_) => MessageKind::EmitEvent,
            Self::QueryObject(_) => MessageKind::QueryObject,
            Self::QueryObjectReply(_) => MessageKind::QueryObjectReply,
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
            Self::Sync(_) => MessageKind::Sync,
            Self::SyncReply(_) => MessageKind::SyncReply,
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
            Self::SubscribeObjects(msg) => msg.serialize_message(),
            Self::SubscribeObjectsReply(msg) => msg.serialize_message(),
            Self::UnsubscribeObjects(msg) => msg.serialize_message(),
            Self::ObjectCreatedEvent(msg) => msg.serialize_message(),
            Self::ObjectDestroyedEvent(msg) => msg.serialize_message(),
            Self::CreateService(msg) => msg.serialize_message(),
            Self::CreateServiceReply(msg) => msg.serialize_message(),
            Self::DestroyService(msg) => msg.serialize_message(),
            Self::DestroyServiceReply(msg) => msg.serialize_message(),
            Self::SubscribeServices(msg) => msg.serialize_message(),
            Self::SubscribeServicesReply(msg) => msg.serialize_message(),
            Self::UnsubscribeServices(msg) => msg.serialize_message(),
            Self::ServiceCreatedEvent(msg) => msg.serialize_message(),
            Self::ServiceDestroyedEvent(msg) => msg.serialize_message(),
            Self::CallFunction(msg) => msg.serialize_message(),
            Self::CallFunctionReply(msg) => msg.serialize_message(),
            Self::SubscribeEvent(msg) => msg.serialize_message(),
            Self::SubscribeEventReply(msg) => msg.serialize_message(),
            Self::UnsubscribeEvent(msg) => msg.serialize_message(),
            Self::EmitEvent(msg) => msg.serialize_message(),
            Self::QueryObject(msg) => msg.serialize_message(),
            Self::QueryObjectReply(msg) => msg.serialize_message(),
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
            Self::Sync(msg) => msg.serialize_message(),
            Self::SyncReply(msg) => msg.serialize_message(),
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
            MessageKind::SubscribeObjects => {
                SubscribeObjects::deserialize_message(buf).map(Self::SubscribeObjects)
            }
            MessageKind::SubscribeObjectsReply => {
                SubscribeObjectsReply::deserialize_message(buf).map(Self::SubscribeObjectsReply)
            }
            MessageKind::UnsubscribeObjects => {
                UnsubscribeObjects::deserialize_message(buf).map(Self::UnsubscribeObjects)
            }
            MessageKind::ObjectCreatedEvent => {
                ObjectCreatedEvent::deserialize_message(buf).map(Self::ObjectCreatedEvent)
            }
            MessageKind::ObjectDestroyedEvent => {
                ObjectDestroyedEvent::deserialize_message(buf).map(Self::ObjectDestroyedEvent)
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
            MessageKind::SubscribeServices => {
                SubscribeServices::deserialize_message(buf).map(Self::SubscribeServices)
            }
            MessageKind::SubscribeServicesReply => {
                SubscribeServicesReply::deserialize_message(buf).map(Self::SubscribeServicesReply)
            }
            MessageKind::UnsubscribeServices => {
                UnsubscribeServices::deserialize_message(buf).map(Self::UnsubscribeServices)
            }
            MessageKind::ServiceCreatedEvent => {
                ServiceCreatedEvent::deserialize_message(buf).map(Self::ServiceCreatedEvent)
            }
            MessageKind::ServiceDestroyedEvent => {
                ServiceDestroyedEvent::deserialize_message(buf).map(Self::ServiceDestroyedEvent)
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
            MessageKind::QueryObject => {
                QueryObject::deserialize_message(buf).map(Self::QueryObject)
            }
            MessageKind::QueryObjectReply => {
                QueryObjectReply::deserialize_message(buf).map(Self::QueryObjectReply)
            }
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
            MessageKind::Sync => Sync::deserialize_message(buf).map(Self::Sync),
            MessageKind::SyncReply => SyncReply::deserialize_message(buf).map(Self::SyncReply),
        }
    }

    fn value(&self) -> Option<&SerializedValue> {
        match self {
            Self::Connect(msg) => msg.value(),
            Self::ConnectReply(msg) => msg.value(),
            Self::Shutdown(msg) => msg.value(),
            Self::CreateObject(msg) => msg.value(),
            Self::CreateObjectReply(msg) => msg.value(),
            Self::DestroyObject(msg) => msg.value(),
            Self::DestroyObjectReply(msg) => msg.value(),
            Self::SubscribeObjects(msg) => msg.value(),
            Self::SubscribeObjectsReply(msg) => msg.value(),
            Self::UnsubscribeObjects(msg) => msg.value(),
            Self::ObjectCreatedEvent(msg) => msg.value(),
            Self::ObjectDestroyedEvent(msg) => msg.value(),
            Self::CreateService(msg) => msg.value(),
            Self::CreateServiceReply(msg) => msg.value(),
            Self::DestroyService(msg) => msg.value(),
            Self::DestroyServiceReply(msg) => msg.value(),
            Self::SubscribeServices(msg) => msg.value(),
            Self::SubscribeServicesReply(msg) => msg.value(),
            Self::UnsubscribeServices(msg) => msg.value(),
            Self::ServiceCreatedEvent(msg) => msg.value(),
            Self::ServiceDestroyedEvent(msg) => msg.value(),
            Self::CallFunction(msg) => msg.value(),
            Self::CallFunctionReply(msg) => msg.value(),
            Self::SubscribeEvent(msg) => msg.value(),
            Self::SubscribeEventReply(msg) => msg.value(),
            Self::UnsubscribeEvent(msg) => msg.value(),
            Self::EmitEvent(msg) => msg.value(),
            Self::QueryObject(msg) => msg.value(),
            Self::QueryObjectReply(msg) => msg.value(),
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
            Self::Sync(msg) => msg.value(),
            Self::SyncReply(msg) => msg.value(),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OptionKind {
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

    fn put_bool(&mut self, v: bool) {
        self.buf.put_u8(v as u8);
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

    fn try_get_bool(&mut self) -> Result<bool, MessageDeserializeError> {
        self.buf.try_get_u8().map(|v| v != 0)
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
