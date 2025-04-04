#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

impl<T: PrimaryTag> PrimaryTag for Option<T> {
    type Tag = tags::Option<T::Tag>;
}

impl<T: Tag, U: Serialize<T>> Serialize<tags::Option<T>> for Option<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Some(value) => serializer.serialize_some(value),
            Self::None => serializer.serialize_none(),
        }
    }

    fn serializes_as_some(&self) -> bool {
        self.is_some()
    }
}

impl<'a, T, U> Serialize<tags::Option<T>> for &'a Option<U>
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Option::Some(value) => serializer.serialize_some(value),
            Option::None => serializer.serialize_none(),
        }
    }

    fn serializes_as_some(&self) -> bool {
        self.is_some()
    }
}

impl<T: Tag, U: Deserialize<T>> Deserialize<tags::Option<T>> for Option<U> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_option()
    }
}

impl<T: Tag> Serialize<tags::Option<T>> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_none()
    }

    fn serializes_as_some(&self) -> bool {
        false
    }
}

impl<T: Tag> Serialize<tags::Option<T>> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_none()
    }

    fn serializes_as_some(&self) -> bool {
        false
    }
}

impl<T: Tag> Deserialize<tags::Option<T>> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_none()
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for Option<T> {
    fn layout() -> Layout {
        BuiltInType::Option(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::option(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}
