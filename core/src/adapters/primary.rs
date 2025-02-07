use crate::tags::{KeyTag, KeyTagImpl, PrimaryKeyTag, PrimaryTag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};

#[derive(Debug)]
pub struct AsPrimary<T>(pub T);

impl<T: PrimaryTag> PrimaryTag for AsPrimary<T> {
    type Tag = T::Tag;
}

impl<T: PrimaryTag + Serialize<T::Tag>> Serialize<T::Tag> for AsPrimary<T> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(self.0)
    }
}

impl<'a, T> Serialize<T::Tag> for &'a AsPrimary<T>
where
    T: PrimaryTag,
    &'a T: Serialize<T::Tag>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self.0)
    }
}

impl<T: PrimaryTag + Deserialize<T::Tag>> Deserialize<T::Tag> for AsPrimary<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize().map(Self)
    }
}

impl<T: PrimaryKeyTag> PrimaryKeyTag for AsPrimary<T> {
    type KeyTag = T::KeyTag;
}

impl<T: PrimaryKeyTag + SerializeKey<T::KeyTag>> SerializeKey<T::KeyTag> for AsPrimary<T> {
    fn try_as_key(
        &self,
    ) -> Result<<<T::KeyTag as KeyTag>::Impl as KeyTagImpl>::Key<'_>, SerializeError> {
        self.0.try_as_key()
    }
}

impl<T: PrimaryKeyTag + DeserializeKey<T::KeyTag>> DeserializeKey<T::KeyTag> for AsPrimary<T> {
    fn try_from_key(
        key: <<T::KeyTag as KeyTag>::Impl as KeyTagImpl>::Key<'_>,
    ) -> Result<Self, DeserializeError> {
        T::try_from_key(key).map(Self)
    }
}
