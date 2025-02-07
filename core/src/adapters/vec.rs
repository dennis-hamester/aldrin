use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};

#[derive(Debug)]
pub struct IterAsVec<T>(pub T);

impl<T> PrimaryTag for IterAsVec<T>
where
    T: IntoIterator,
    T::Item: PrimaryTag,
{
    type Tag = tags::Vec<<T::Item as PrimaryTag>::Tag>;
}

impl<T, U> Serialize<tags::Vec<T>> for IterAsVec<U>
where
    T: Tag,
    U: IntoIterator,
    U::IntoIter: ExactSizeIterator,
    U::Item: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self.0)
    }
}

impl<'a, T, U> Serialize<tags::Vec<T>> for &'a IterAsVec<U>
where
    T: Tag,
    &'a U: IntoIterator,
    <&'a U as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'a U as IntoIterator>::Item: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(&self.0)
    }
}
