#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_op_in_unsafe_fn)]

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
pub use value::{ByteSlice, Bytes, Skip, ValueKind};
pub use value_deserializer::{
    BytesDeserializer, Deserialize, Deserializer, ElementDeserializer, EnumDeserializer,
    FieldDeserializer, MapDeserializer, SetDeserializer, StructDeserializer, VecDeserializer,
};
pub use value_serializer::{
    BytesSerializer, MapSerializer, Serialize, Serializer, SetSerializer, StructSerializer,
    VecSerializer,
};

const MAX_VALUE_DEPTH: u8 = 32;
