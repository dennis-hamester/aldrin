use crate::{DeserializeError, Deserializer, Tag};

pub trait Deserialize<T: Tag>: Sized {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError>;
}
