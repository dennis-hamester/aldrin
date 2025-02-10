#![deny(missing_debug_implementations)]

mod buf_ext;
mod bytes;
mod deserialize;
mod deserializer;
mod error;
mod ids;
mod impls;
mod key_tag;
mod primary_tag;
mod protocol_version;
mod serialize;
mod serialize_key;
mod serialized_value;
mod serializer;
mod tag;
mod value;
mod value_kind;
// mod bus_listener;
// mod channel_end;
// mod deserialize_key;
// mod generic_value;
// mod message_deserializer;
// mod message_serializer;
// mod serialize_key;
// mod service_info;
// mod unknown_fields;
// mod unknown_variant;
// mod value;
// mod value_deserializer;
// mod value_serializer;

// #[cfg(feature = "channel")]
// pub mod channel;
// #[cfg(feature = "introspection")]
// pub mod introspection;
// pub mod message;
// pub mod test;
// #[cfg(feature = "tokio")]
// pub mod tokio;
// pub mod transport;

pub use bytes::{ByteSlice, Bytes};
pub use deserialize::Deserialize;
pub use deserializer::{Deserializer, VecDeserializer};
pub use error::{
    DeserializeError, MessageDeserializeError, MessageSerializeError, ProtocolVersionError,
    SerializeError,
};
pub use ids::{
    BusListenerCookie, ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid, TypeId,
};
pub use key_tag::KeyTag;
pub use primary_tag::PrimaryTag;
pub use protocol_version::ProtocolVersion;
pub use serialize::Serialize;
pub use serialize_key::SerializeKey;
pub use serialized_value::{SerializedValue, SerializedValueSlice};
pub use serializer::{Serializer, VecSerializer};
pub use tag::{Map, Receiver, Sender, Set, Tag};
pub use value::Value;
pub use value_kind::ValueKind;
// #[cfg(feature = "derive")]
// pub use aldrin_macros::{AsSerializeArg, Deserialize, DeserializeKey, Serialize, SerializeKey};
// #[cfg(all(feature = "derive", feature = "introspection"))]
// pub use aldrin_macros::{Introspectable, KeyTypeOf};
// pub use bus_listener::{BusEvent, BusListenerFilter, BusListenerScope, BusListenerServiceFilter};
// pub use channel_end::{ChannelEnd, ChannelEndWithCapacity};
// pub use deserialize_key::{DeserializeKey, DeserializeKeyImpl};
// pub use generic_value::{Enum, Struct, Value};
// pub use serialize_key::{SerializeKey, SerializeKeyImpl};
// pub use service_info::ServiceInfo;
// pub use unknown_fields::UnknownFields;
// pub use unknown_variant::UnknownVariant;
// pub use value::{ByteSlice, Bytes, Skip, ValueKind};
// pub use value_deserializer::{
//     BytesDeserializer, Deserialize, Deserializer, ElementDeserializer, EnumDeserializer,
//     FieldDeserializer, MapDeserializer, SetDeserializer, StructDeserializer, VecDeserializer,
// };
// pub use value_serializer::{
//     AsSerializeArg, BytesSerializer, MapSerializer, Serialize, SerializeArg, Serializer,
//     SetSerializer, StructSerializer, VecSerializer,
// };

const MAX_VALUE_DEPTH: u8 = 32;
