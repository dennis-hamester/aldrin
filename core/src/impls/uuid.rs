#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, PrimaryTag, Receiver, Sender, Tag};
use crate::{
    ChannelCookie, Deserialize, DeserializeError, Deserializer, Serialize, SerializeError,
    Serializer,
};
use uuid::Uuid;

impl PrimaryTag for Uuid {
    type Tag = tags::Uuid;
}

impl Serialize<tags::Uuid> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self)
    }
}

impl Serialize<tags::Uuid> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid()
    }
}

impl<T: Tag> Serialize<Sender<T>> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(ChannelCookie(self))
    }
}

impl<T: Tag> Serialize<Sender<T>> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Sender<T>, _>(*self)
    }
}

impl<T: Tag> Deserialize<Sender<T>> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(|cookie| cookie.0)
    }
}

impl<T: Tag> Serialize<Receiver<T>> for Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(ChannelCookie(self))
    }
}

impl<T: Tag> Serialize<Receiver<T>> for &Uuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Receiver<T>, _>(*self)
    }
}

impl<T: Tag> Deserialize<Receiver<T>> for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(|cookie| cookie.0)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for Uuid {
    fn layout() -> Layout {
        BuiltInType::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}
