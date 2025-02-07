use crate::tags::{KeyTag, Map, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, SerializeKey, Serializer};

#[derive(Debug)]
pub struct IterAsMap<T>(pub T);

impl<K, T, I> PrimaryTag for IterAsMap<I>
where
    K: PrimaryKeyTag,
    T: PrimaryTag,
    I: IntoIterator<Item = (K, T)>,
{
    type Tag = Map<K::KeyTag, T::Tag>;
}

impl<K, L, T, U, I> Serialize<Map<K, T>> for IterAsMap<I>
where
    K: KeyTag,
    L: SerializeKey<K>,
    T: Tag,
    U: Serialize<T>,
    I: IntoIterator<Item = (L, U)>,
    I::IntoIter: ExactSizeIterator,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map_iter(self.0)
    }
}

impl<'a, K, L, T, U, I> Serialize<Map<K, T>> for &'a IterAsMap<I>
where
    K: KeyTag,
    L: SerializeKey<K>,
    T: Tag,
    U: Serialize<T>,
    &'a I: IntoIterator<Item = (L, U)>,
    <&'a I as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map_iter(&self.0)
    }
}
