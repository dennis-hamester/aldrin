#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

mod deserialize_key;
mod error;
mod generic_value;
mod ids;
mod serialize_key;
mod util;
mod value;
mod value_deserializer;
mod value_serializer;

pub mod message;
pub mod transport;

pub use deserialize_key::DeserializeKey;
pub use error::{DeserializeError, SerializeError};
pub use generic_value::{Enum, Struct, Value};
pub use ids::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
pub use serialize_key::SerializeKey;
pub use value::{Bytes, BytesRef, ValueKind};
pub use value_deserializer::{
    BytesDeserializer, Deserialize, Deserializer, EnumDeserializer, FieldDeserializer,
    MapDeserializer, SetDeserializer, StructDeserializer, VecDeserializer,
};
pub use value_serializer::{
    MapSerializer, Serialize, Serializer, SetSerializer, StructSerializer, VecSerializer,
};

pub const VERSION: u32 = 12;
