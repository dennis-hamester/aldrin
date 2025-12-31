#[cfg(feature = "introspection")]
use crate::introspection::{Introspectable, LexicalId, References, ir};
use crate::tags::{KeyTag, PrimaryKeyTag, PrimaryTag, Set};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasher, Hash};

impl<K: PrimaryKeyTag, S> PrimaryTag for HashSet<K, S> {
    type Tag = Set<K::KeyTag>;
}

impl<K, T, S> Serialize<Set<K>> for HashSet<T, S>
where
    K: KeyTag,
    T: SerializeKey<K>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2_iter(self)
    }
}

impl<'a, K, T, S> Serialize<Set<K>> for &'a HashSet<T, S>
where
    K: KeyTag,
    &'a T: SerializeKey<K>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2_iter(self)
    }
}

impl<K, T, S> Deserialize<Set<K>> for HashSet<T, S>
where
    K: KeyTag,
    T: DeserializeKey<K> + Eq + Hash,
    S: Default + BuildHasher,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_set_extend_new()
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable, S> Introspectable for HashSet<T, S> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Set(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::set(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>()
    }
}

impl<K: PrimaryKeyTag> PrimaryTag for BTreeSet<K> {
    type Tag = Set<K::KeyTag>;
}

impl<K, T> Serialize<Set<K>> for BTreeSet<T>
where
    K: KeyTag,
    T: SerializeKey<K>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2_iter(self)
    }
}

impl<'a, K, T> Serialize<Set<K>> for &'a BTreeSet<T>
where
    K: KeyTag,
    &'a T: SerializeKey<K>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2_iter(self)
    }
}

impl<K, T> Deserialize<Set<K>> for BTreeSet<T>
where
    K: KeyTag,
    T: DeserializeKey<K> + Ord,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_set_extend_new()
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for BTreeSet<T> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Set(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::set(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>()
    }
}

impl<T: KeyTag> Serialize<Set<T>> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2::<T>()?.finish()
    }
}

impl<T: KeyTag> Serialize<Set<T>> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set2::<T>()?.finish()
    }
}

impl<T: KeyTag> Deserialize<Set<T>> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_set::<T>()?.finish(())
    }
}
