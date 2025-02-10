use crate::{SerializeError, Serializer, Tag};

pub trait Serialize<T: Tag>: Sized {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError>;
}
