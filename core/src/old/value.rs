#[cfg(test)]
mod test;

use crate::deserialize_key::DeserializeKey;
use crate::error::{DeserializeError, SerializeError};
#[cfg(feature = "introspection")]
use crate::introspection::{
    ArrayType, BuiltInType, DynIntrospectable, Introspectable, KeyTypeOf, Layout, LexicalId,
    MapType, References, ResultType, Struct,
};
use crate::serialize_key::SerializeKey;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{AsSerializeArg, Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::convert::Infallible;
use std::hash::{BuildHasher, Hash};
use std::mem::MaybeUninit;
use std::ops::Deref;
use uuid::Uuid;

/// Empty value that deserializes from everything by skipping over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skip;

impl Deserialize for Skip {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.skip().map(|_| Self)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for &T {
    fn layout() -> Layout {
        BuiltInType::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for &mut T {
    fn layout() -> Layout {
        BuiltInType::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for Box<T> {
    fn layout() -> Layout {
        BuiltInType::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for () {
    fn layout() -> Layout {
        BuiltInType::Unit.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UNIT
    }

    fn add_references(_references: &mut References) {}
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

#[cfg(feature = "introspection")]
impl Introspectable for bool {
    fn layout() -> Layout {
        BuiltInType::Bool.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BOOL
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for u8 {
    fn layout() -> Layout {
        BuiltInType::U8.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::U8
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for i8 {
    fn layout() -> Layout {
        BuiltInType::I8.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::I8
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for u16 {
    fn layout() -> Layout {
        BuiltInType::U16.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::U16
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for i16 {
    fn layout() -> Layout {
        BuiltInType::I16.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::I16
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for u32 {
    fn layout() -> Layout {
        BuiltInType::U32.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::U32
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for i32 {
    fn layout() -> Layout {
        BuiltInType::I32.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::I32
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for u64 {
    fn layout() -> Layout {
        BuiltInType::U64.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::U64
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for i64 {
    fn layout() -> Layout {
        BuiltInType::I64.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::I64
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for f32 {
    fn layout() -> Layout {
        BuiltInType::F32.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::F32
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for f64 {
    fn layout() -> Layout {
        BuiltInType::F64.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::F64
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for str {
    fn layout() -> Layout {
        BuiltInType::String.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::STRING
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for String {
    fn layout() -> Layout {
        BuiltInType::String.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::STRING
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl Introspectable for Uuid {
    fn layout() -> Layout {
        BuiltInType::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for Vec<T> {
    fn layout() -> Layout {
        BuiltInType::Vec(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::vec(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for VecDeque<T> {
    fn layout() -> Layout {
        BuiltInType::Vec(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::vec(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for LinkedList<T> {
    fn layout() -> Layout {
        BuiltInType::Vec(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::vec(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for [T] {
    fn layout() -> Layout {
        BuiltInType::Vec(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::vec(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable, const N: usize> Introspectable for [T; N] {
    fn layout() -> Layout {
        BuiltInType::Array(ArrayType::new(T::lexical_id(), N as u32)).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::array(T::lexical_id(), N as u32)
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

impl Serialize for bytes::Bytes {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize for bytes::Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_to_vec().map(Self::from)
    }
}

impl AsSerializeArg for bytes::Bytes {
    type SerializeArg<'a> = &'a ByteSlice;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        ByteSlice::new(self)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::Bytes {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

impl Serialize for bytes::BytesMut {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize for bytes::BytesMut {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        // This is inefficient, but bytes doesn't currently offer a better alternative.
        let vec = deserializer.deserialize_bytes_to_vec()?;
        Ok(Self::from(vec.as_slice()))
    }
}

impl AsSerializeArg for bytes::BytesMut {
    type SerializeArg<'a> = &'a ByteSlice;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        ByteSlice::new(self)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::BytesMut {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

impl<K: SerializeKey, V: Serialize, S> Serialize for HashMap<K, V, S> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map_iter(self)
    }
}

impl<K, V, S> Deserialize for HashMap<K, V, S>
where
    K: DeserializeKey + Eq + Hash,
    V: Deserialize,
    S: BuildHasher + Default,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_map_extend_new()
    }
}

impl<K: SerializeKey, V: Serialize, S> AsSerializeArg for HashMap<K, V, S> {
    type SerializeArg<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
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

impl<K: SerializeKey, V: Serialize> Serialize for BTreeMap<K, V> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_map_iter(self)
    }
}

impl<K: DeserializeKey + Ord, V: Deserialize> Deserialize for BTreeMap<K, V> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_map_extend_new()
    }
}

impl<K: SerializeKey, V: Serialize> AsSerializeArg for BTreeMap<K, V> {
    type SerializeArg<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
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

impl<T: SerializeKey, S> Serialize for HashSet<T, S> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set_iter(self)
    }
}

impl<T, S> Deserialize for HashSet<T, S>
where
    T: DeserializeKey + Eq + Hash,
    S: BuildHasher + Default,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_set_extend_new()
    }
}

impl<T: SerializeKey, S> AsSerializeArg for HashSet<T, S> {
    type SerializeArg<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[cfg(feature = "introspection")]
impl<T: KeyTypeOf, S> Introspectable for HashSet<T, S> {
    fn layout() -> Layout {
        BuiltInType::Set(T::KEY_TYPE).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::set(T::KEY_TYPE)
    }

    fn add_references(_references: &mut References) {}
}

impl<T: SerializeKey> Serialize for BTreeSet<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set_iter(self)
    }
}

impl<T: DeserializeKey + Ord> Deserialize for BTreeSet<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_set_extend_new()
    }
}

impl<T: SerializeKey> AsSerializeArg for BTreeSet<T> {
    type SerializeArg<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[cfg(feature = "introspection")]
impl<T: KeyTypeOf> Introspectable for BTreeSet<T> {
    fn layout() -> Layout {
        BuiltInType::Set(T::KEY_TYPE).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::set(T::KEY_TYPE)
    }

    fn add_references(_references: &mut References) {}
}

impl<'a, T> Serialize for Cow<'a, T>
where
    T: Serialize + ToOwned + ?Sized + 'a,
{
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Cow::Borrowed(borrowed) => borrowed.serialize(serializer),
            Cow::Owned(owned) => owned.borrow().serialize(serializer),
        }
    }
}

impl<'a, T> Deserialize for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'a,
    T::Owned: Deserialize,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        T::Owned::deserialize(deserializer).map(Self::Owned)
    }
}

impl<'a, T> AsSerializeArg for Cow<'a, T>
where
    T: AsSerializeArg + ToOwned + ?Sized + 'a,
{
    type SerializeArg<'b>
        = T::SerializeArg<'b>
    where
        Self: 'b;

    fn as_serialize_arg<'b>(&'b self) -> Self::SerializeArg<'b>
    where
        Self: 'b,
    {
        self.as_ref().as_serialize_arg()
    }
}

#[cfg(feature = "introspection")]
impl<'a, T> Introspectable for Cow<'a, T>
where
    T: Introspectable + ToOwned + ?Sized + 'a,
{
    fn layout() -> Layout {
        T::layout()
    }

    fn lexical_id() -> LexicalId {
        T::lexical_id()
    }

    fn add_references(_references: &mut References) {}
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultVariant {
    Ok = 0,
    Err = 1,
}

impl<T, E> Serialize for Result<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Ok(ok) => serializer.serialize_enum(ResultVariant::Ok, ok),
            Err(err) => serializer.serialize_enum(ResultVariant::Err, err),
        }
    }
}

impl<T, E> Deserialize for Result<T, E>
where
    T: Deserialize,
    E: Deserialize,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            ResultVariant::Ok => deserializer.deserialize().map(Ok),
            ResultVariant::Err => deserializer.deserialize().map(Err),
        }
    }
}

impl<T: AsSerializeArg, E: AsSerializeArg> AsSerializeArg for Result<T, E> {
    type SerializeArg<'a>
        = Result<T::SerializeArg<'a>, E::SerializeArg<'a>>
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self.as_ref()
            .map(T::as_serialize_arg)
            .map_err(E::as_serialize_arg)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable, E: Introspectable> Introspectable for Result<T, E> {
    fn layout() -> Layout {
        BuiltInType::Result(ResultType::new(T::lexical_id(), E::lexical_id())).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::result(T::lexical_id(), E::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
        references.add::<E>();
    }
}

impl Serialize for Infallible {
    fn serialize(&self, _serializer: Serializer) -> Result<(), SerializeError> {
        match *self {}
    }
}

impl Deserialize for Infallible {
    fn deserialize(_deserializer: Deserializer) -> Result<Self, DeserializeError> {
        Err(DeserializeError::UnexpectedValue)
    }
}

impl AsSerializeArg for Infallible {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
}

macro_rules! tuple_impls {
    { $len:literal, $( ($gen:ident, $idx:tt) ),+ } => {
        impl<$( $gen ),+> Serialize for ($( $gen, )+)
        where
            $( $gen: Serialize ),+
        {
            fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
                let mut serializer = serializer.serialize_struct($len)?;

                $(
                    serializer.serialize_field($idx as u32, &self.$idx)?;
                )+

                serializer.finish()
            }
        }

        impl<$( $gen ),+> Deserialize for ($( $gen, )+)
        where
            $( $gen: Deserialize ),+
        {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                let mut deserializer = deserializer.deserialize_struct()?;

                $(
                    #[allow(non_snake_case)]
                    let mut $gen = None;
                )+

                while deserializer.has_more_fields() {
                    let deserializer = deserializer.deserialize_field()?;

                    match deserializer.id() {
                        $( $idx => $gen = deserializer.deserialize().map(Some)?, )+
                        _ => deserializer.skip()?,
                    }
                }

                deserializer.finish_with(|_| {
                    Ok(($( $gen.ok_or(DeserializeError::InvalidSerialization)?, )+))
                })
            }
        }

        impl<$( $gen ),+> AsSerializeArg for ($( $gen, )+)
        where
            $( $gen: AsSerializeArg ),+
        {
            type SerializeArg<'a>
                = ($( $gen::SerializeArg<'a>, )+)
            where
                Self: 'a;

            fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
            where
                Self: 'a,
            {
                ($( self.$idx.as_serialize_arg(), )+)
            }
        }

        #[cfg(feature = "introspection")]
        impl<$( $gen ),+> Introspectable for ($( $gen, )+)
        where
            $( $gen: Introspectable ),+
        {
            fn layout() -> Layout {
                Struct::builder("std", concat!("Tuple", $len))
                    $( .field($idx, concat!("field", $idx), true, $gen::lexical_id()) )+
                    .finish()
                    .into()
            }

            fn lexical_id() -> LexicalId {
                LexicalId::custom_generic(
                    "std",
                    concat!("Tuple", $len),
                    &[$( $gen::lexical_id() ),+],
                )
            }

            fn add_references(references: &mut References) {
                let types: [DynIntrospectable; $len] = [
                    $( DynIntrospectable::new::<$gen>() ),+
                ];

                references.extend(types);
            }
        }
    };
}

tuple_impls! {  1, (T0, 0) }
tuple_impls! {  2, (T0, 0), (T1, 1) }
tuple_impls! {  3, (T0, 0), (T1, 1), (T2, 2) }
tuple_impls! {  4, (T0, 0), (T1, 1), (T2, 2), (T3, 3) }
tuple_impls! {  5, (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4) }
tuple_impls! {  6, (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5) }
tuple_impls! {  7, (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6) }
tuple_impls! {  8, (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7) }
tuple_impls! {  9, (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8) }
tuple_impls! { 10,
    (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9)
}
tuple_impls! { 11,
    (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9),
    (T10, 10)
}
tuple_impls! { 12,
    (T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8),
    (T9, 9), (T10, 10), (T11, 11)
}
