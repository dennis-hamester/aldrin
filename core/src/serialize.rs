use crate::tags::Tag;
use crate::{SerializeError, Serializer};

pub trait Serialize<T: Tag>: Sized {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError>;
}
