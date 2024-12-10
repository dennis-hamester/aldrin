#![deny(missing_debug_implementations)]

mod buf_ext;
mod bus_listener;
mod channel_end;
mod deserialize_key;
mod error;
mod generic_value;
mod ids;
mod message_deserializer;
mod message_serializer;
mod protocol_version;
mod serialize_key;
mod serialized_value;
mod service_info;
mod unknown_variant;
mod value;
mod value_deserializer;
mod value_serializer;

#[cfg(feature = "channel")]
pub mod channel;
#[cfg(feature = "introspection")]
pub mod introspection;
pub mod message;
#[cfg(feature = "tokio")]
pub mod tokio;
pub mod transport;

#[cfg(feature = "derive")]
pub use aldrin_macros::{AsSerializeArg, Deserialize, DeserializeKey, Serialize, SerializeKey};
#[cfg(all(feature = "derive", feature = "introspection"))]
pub use aldrin_macros::{Introspectable, KeyTypeOf};
pub use bus_listener::{BusEvent, BusListenerFilter, BusListenerScope, BusListenerServiceFilter};
pub use channel_end::{ChannelEnd, ChannelEndWithCapacity};
pub use deserialize_key::{DeserializeKey, DeserializeKeyImpl};
pub use error::{DeserializeError, ProtocolVersionError, SerializeError};
pub use generic_value::{Enum, Struct, Value};
pub use ids::{
    BusListenerCookie, ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid, TypeId,
};
pub use protocol_version::ProtocolVersion;
pub use serialize_key::{SerializeKey, SerializeKeyImpl};
pub use serialized_value::{SerializedValue, SerializedValueSlice};
pub use service_info::ServiceInfo;
pub use unknown_variant::UnknownVariant;
pub use value::{ByteSlice, Bytes, Skip, ValueKind};
pub use value_deserializer::{
    BytesDeserializer, Deserialize, Deserializer, ElementDeserializer, EnumDeserializer,
    FieldDeserializer, FieldWithFallbackDeserializer, MapDeserializer, SetDeserializer,
    StructDeserializer, StructWithFallbackDeserializer, VecDeserializer,
};
pub use value_serializer::{
    AsSerializeArg, BytesSerializer, MapSerializer, Serialize, SerializeArg, Serializer,
    SetSerializer, StructSerializer, VecSerializer,
};

const MAX_VALUE_DEPTH: u8 = 32;
