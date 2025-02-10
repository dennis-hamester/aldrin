use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Tag, Value,
};

impl<T: PrimaryTag> PrimaryTag for Option<T> {
    type Tag = Option<T::Tag>;
}

impl<T: Tag, U: Serialize<T>> Serialize<Option<T>> for Option<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Some(value) => serializer.serialize_some(value),
            Self::None => serializer.serialize_none(),
        }
    }
}

impl<T: Tag, U: Deserialize<T>> Deserialize<Option<T>> for Option<U> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_option()
    }
}

impl<'a, T, U> Serialize<Option<T>> for &'a Option<U>
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
}

impl<T: Serialize<Value>> Serialize<Value> for Option<T> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Some(value) => serializer.serialize_some(value),
            Self::None => serializer.serialize_none(),
        }
    }
}

impl<T: Deserialize<Value>> Deserialize<Value> for Option<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_option()
    }
}

impl<'a, T> Serialize<Value> for &'a Option<T>
where
    &'a T: Serialize<Value>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Option::Some(value) => serializer.serialize_some(value),
            Option::None => serializer.serialize_none(),
        }
    }
}
