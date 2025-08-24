use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use std::convert::Infallible;

#[derive(Debug, Copy, Clone)]
pub struct AsOk<T>(pub T);

impl<T: PrimaryTag> PrimaryTag for AsOk<T> {
    type Tag = Result<T::Tag, tags::Value>;
}

impl<T: Tag, E: Tag, U: Serialize<T>> Serialize<Result<T, E>> for AsOk<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Result<_, E>>(Ok::<_, Infallible>(self.0))
    }
}

impl<'a, T, E, U> Serialize<Result<T, E>> for &'a AsOk<U>
where
    T: Tag,
    E: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Result<_, E>>(Ok::<_, Infallible>(&self.0))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AsErr<T>(pub T);

impl<T: PrimaryTag> PrimaryTag for AsErr<T> {
    type Tag = Result<tags::Value, T::Tag>;
}

impl<T: Tag, E: Tag, F: Serialize<E>> Serialize<Result<T, E>> for AsErr<F> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Result<T, _>>(Err::<Infallible, _>(self.0))
    }
}

impl<'a, T, E, F> Serialize<Result<T, E>> for &'a AsErr<F>
where
    T: Tag,
    E: Tag,
    &'a F: Serialize<E>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Result<T, _>>(Err::<Infallible, _>(&self.0))
    }
}
