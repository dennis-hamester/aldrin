use crate::tags::{KeyTag, PrimaryKeyTag, PrimaryTag, Set};
use crate::{Serialize, SerializeError, SerializeKey, Serializer};

#[derive(Debug)]
pub struct IterAsSet<T>(pub T);

impl<T> PrimaryTag for IterAsSet<T>
where
    T: IntoIterator,
    T::Item: PrimaryKeyTag,
{
    type Tag = Set<<T::Item as PrimaryKeyTag>::KeyTag>;
}

impl<T, U> Serialize<Set<T>> for IterAsSet<U>
where
    T: KeyTag,
    U: IntoIterator,
    U::IntoIter: ExactSizeIterator,
    U::Item: SerializeKey<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set_iter(self.0)
    }
}

impl<'a, T, U> Serialize<Set<T>> for &'a IterAsSet<U>
where
    T: KeyTag,
    &'a U: IntoIterator,
    <&'a U as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'a U as IntoIterator>::Item: SerializeKey<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set_iter(&self.0)
    }
}
