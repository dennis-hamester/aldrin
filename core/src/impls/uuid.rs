use crate::{
    ChannelCookie, Deserialize, DeserializeError, Deserializer, PrimaryTag, Receiver, Sender,
    Serialize, SerializeError, Serializer, Tag, Value, ValueKind,
};
use uuid::Uuid;

impl PrimaryTag for Uuid {
    type Tag = Self;
}

impl Serialize<Self> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self)
    }
}

impl Deserialize<Self> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid()
    }
}

impl Serialize<Uuid> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Uuid, _>(*self)
    }
}

impl Serialize<Value> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self)
    }
}

impl Deserialize<Value> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::Uuid => deserializer.deserialize_uuid(),
            ValueKind::Sender => deserializer.deserialize_sender().map(|cookie| cookie.0),
            ValueKind::Receiver => deserializer.deserialize_receiver().map(|cookie| cookie.0),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl Serialize<Value> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Value, _>(*self)
    }
}

impl<T: Tag> Serialize<Sender<T>> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(ChannelCookie(self))
    }
}

impl<T: Tag> Deserialize<Sender<T>> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(|cookie| cookie.0)
    }
}

impl<T: Tag> Serialize<Sender<T>> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Sender<T>, _>(*self)
    }
}

impl<T: Tag> Serialize<Receiver<T>> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(ChannelCookie(self))
    }
}

impl<T: Tag> Deserialize<Receiver<T>> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(|cookie| cookie.0)
    }
}

impl<T: Tag> Serialize<Receiver<T>> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Receiver<T>, _>(*self)
    }
}
