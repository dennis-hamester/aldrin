mod serialize_key;
mod serializer;

use crate::{SerializeError, Tag};

pub use serialize_key::SerializeKey;
pub use serializer::Serializer;

pub trait Serialize<T: Tag>: Sized {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError>;
}
