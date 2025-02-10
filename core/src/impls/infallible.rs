use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Tag, Value,
};
use std::convert::Infallible;

impl PrimaryTag for Infallible {
    type Tag = Value;
}

impl<T: Tag> Serialize<T> for Infallible {
    fn serialize(self, _serializer: Serializer) -> Result<(), SerializeError> {
        match self {}
    }
}

impl<T: Tag> Deserialize<T> for Infallible {
    fn deserialize(_deserializer: Deserializer) -> Result<Self, DeserializeError> {
        Err(DeserializeError::UnexpectedValue)
    }
}

impl<T: Tag> Serialize<T> for &Infallible {
    fn serialize(self, _serializer: Serializer) -> Result<(), SerializeError> {
        match *self {}
    }
}

impl<T: Tag> Deserialize<T> for &Infallible {
    fn deserialize(_deserializer: Deserializer) -> Result<Self, DeserializeError> {
        Err(DeserializeError::UnexpectedValue)
    }
}
