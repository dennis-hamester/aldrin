#![deny(missing_debug_implementations)]

mod buf_ext;
mod bus_listener;
mod bytes;
mod channel_end;
mod convert_value;
mod deserialize;
mod deserialize_key;
mod deserializer;
mod ids;
mod impls;
mod key_impls;
mod protocol_version;
mod serialize;
mod serialize_key;
mod serialized_value;
mod serializer;
mod service_info;
mod unknown_fields;
mod unknown_variant;
mod value;
mod value_kind;

pub mod adapters;
#[cfg(feature = "channel")]
pub mod channel;
#[cfg(feature = "introspection")]
pub mod introspection;
pub mod message;
pub mod tags;
#[cfg(feature = "tokio")]
pub mod tokio;
pub mod transport;

pub use crate::bytes::{ByteSlice, Bytes};
#[cfg(all(feature = "derive", feature = "introspection"))]
pub use aldrin_macros::Introspectable;
#[cfg(feature = "derive")]
pub use aldrin_macros::{Deserialize, PrimaryTag, RefType, Serialize, Tag};
pub use bus_listener::{BusEvent, BusListenerFilter, BusListenerScope, BusListenerServiceFilter};
pub use channel_end::{ChannelEnd, ChannelEndWithCapacity};
pub use convert_value::ValueConversionError;
pub use deserialize::{Deserialize, DeserializeError};
pub use deserialize_key::DeserializeKey;
pub use deserializer::{
    Bytes1Deserializer, Bytes2Deserializer, BytesDeserializer, Deserializer, EnumDeserializer,
    FieldDeserializer, Map1Deserializer, Map2Deserializer, MapDeserializer, MapElementDeserializer,
    Set1Deserializer, Set2Deserializer, SetDeserializer, Struct1Deserializer, Struct2Deserializer,
    StructDeserializer, Vec1Deserializer, Vec2Deserializer, VecDeserializer,
};
pub use ids::{
    BusListenerCookie, ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid, TypeId,
};
pub use protocol_version::ProtocolVersion;
pub use serialize::{Serialize, SerializeError};
pub use serialize_key::SerializeKey;
pub use serialized_value::{SerializedValue, SerializedValueSlice};
pub use serializer::{
    Bytes1Serializer, Bytes2Serializer, Map1Serializer, Map2Serializer, Serializer, Set1Serializer,
    Set2Serializer, Struct1Serializer, Struct2Serializer, Vec1Serializer, Vec2Serializer,
};
pub use service_info::ServiceInfo;
pub use unknown_fields::{AsUnknownFields, UnknownFields, UnknownFieldsRef};
pub use unknown_variant::{AsUnknownVariant, UnknownVariant, UnknownVariantRef};
pub use value::{Enum, Struct, Value};
pub use value_kind::ValueKind;

const MAX_VALUE_DEPTH: u8 = 32;
