#[cfg(feature = "introspection")]
use crate::introspection::{
    BuiltInType, Introspectable, KeyTypeOf, Layout, LexicalId, MapType, References,
};
use crate::tags::{KeyTag, Map, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

impl<K: PrimaryKeyTag, T: PrimaryTag, S> PrimaryTag for HashMap<K, T, S> {
    type Tag = Map<K::KeyTag, T::Tag>;
}

impl<K, L, T, U, S> Serialize<Map<K, T>> for HashMap<L, U, S>
where
    K: KeyTag,
    L: SerializeKey<K>,
    T: Tag,
    U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2_iter(self)
    }
}

impl<'a, K, L, T, U, S> Serialize<Map<K, T>> for &'a HashMap<L, U, S>
where
    K: KeyTag,
    &'a L: SerializeKey<K>,
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2_iter(self)
    }
}

impl<K, L, T, U, S> Deserialize<Map<K, T>> for HashMap<L, U, S>
where
    K: KeyTag,
    L: DeserializeKey<K> + Eq + Hash,
    T: Tag,
    U: Deserialize<T>,
    S: Default + BuildHasher,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_map_extend_new()
    }
}

#[cfg(feature = "introspection")]
impl<K, V, S> Introspectable for HashMap<K, V, S>
where
    K: KeyTypeOf,
    V: Introspectable,
{
    fn layout() -> Layout {
        BuiltInType::Map(MapType::new(K::KEY_TYPE, V::lexical_id())).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::map(K::KEY_TYPE, V::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<V>();
    }
}

impl<K: PrimaryKeyTag, T: PrimaryTag> PrimaryTag for BTreeMap<K, T> {
    type Tag = Map<K::KeyTag, T::Tag>;
}

impl<K, L, T, U> Serialize<Map<K, T>> for BTreeMap<L, U>
where
    K: KeyTag,
    L: SerializeKey<K>,
    T: Tag,
    U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2_iter(self)
    }
}

impl<'a, K, L, T, U> Serialize<Map<K, T>> for &'a BTreeMap<L, U>
where
    K: KeyTag,
    &'a L: SerializeKey<K>,
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2_iter(self)
    }
}

impl<K, L, T, U> Deserialize<Map<K, T>> for BTreeMap<L, U>
where
    K: KeyTag,
    L: DeserializeKey<K> + Ord,
    T: Tag,
    U: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_map_extend_new()
    }
}

#[cfg(feature = "introspection")]
impl<K: KeyTypeOf, V: Introspectable> Introspectable for BTreeMap<K, V> {
    fn layout() -> Layout {
        BuiltInType::Map(MapType::new(K::KEY_TYPE, V::lexical_id())).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::map(K::KEY_TYPE, V::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<V>();
    }
}

impl<K: KeyTag, T: Tag> Serialize<Map<K, T>> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2::<K>()?.finish()
    }
}

impl<K: KeyTag, T: Tag> Serialize<Map<K, T>> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map2::<K>()?.finish()
    }
}

impl<K: KeyTag, T: Tag> Deserialize<Map<K, T>> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_map::<K>()?.finish(())
    }
}
