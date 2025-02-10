mod infallible;
mod option;
mod primitive;
mod string;
#[cfg(test)]
mod test;
mod unit;
mod uuid;
mod vec;

use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Tag,
};

impl<T: PrimaryTag + ?Sized> PrimaryTag for &T {
    type Tag = T::Tag;
}

impl<'a, T, U> Serialize<T> for &&'a U
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (*self).serialize(serializer)
    }
}

impl<T: PrimaryTag + ?Sized> PrimaryTag for Box<T> {
    type Tag = T::Tag;
}

impl<T, U> Serialize<T> for Box<U>
where
    T: Tag,
    U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (*self).serialize(serializer)
    }
}

impl<T, U> Deserialize<T> for Box<U>
where
    T: Tag,
    U: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        U::deserialize(deserializer).map(Self::new)
    }
}

impl<'a, T, U> Serialize<T> for &'a Box<U>
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}
