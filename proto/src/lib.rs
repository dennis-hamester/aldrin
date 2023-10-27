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
mod serialize_key;
mod serialized_value;
mod value;
mod value_deserializer;
mod value_serializer;

pub mod message;
#[cfg(feature = "tokio")]
pub mod tokio;
pub mod transport;

pub use bus_listener::BusListenerScope;
pub use deserialize_key::DeserializeKey;
pub use error::{DeserializeError, SerializeError};
pub use generic_value::{Enum, Struct, Value};
pub use ids::{
    BusListenerCookie, ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid,
};
pub use serialize_key::SerializeKey;
pub use serialized_value::{SerializedValue, SerializedValueSlice};
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

pub const VERSION: u32 = 13;
