use crate::tags::Tag;
use crate::{DeserializeError, Deserializer};

pub trait Deserialize<T: Tag>: Sized {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError>;
}
