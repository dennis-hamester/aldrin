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

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum ValueKind {
    None = 0,
    Some = 1,
    Bool = 2,
    U8 = 3,
    I8 = 4,
    U16 = 5,
    I16 = 6,
    U32 = 7,
    I32 = 8,
    U64 = 9,
    I64 = 10,
    F32 = 11,
    F64 = 12,
    String = 13,
    Uuid = 14,
    ObjectId = 15,
    ServiceId = 16,
    Vec = 17,
    Bytes = 18,
    U8Map = 19,
    I8Map = 20,
    U16Map = 21,
    I16Map = 22,
    U32Map = 23,
    I32Map = 24,
    U64Map = 25,
    I64Map = 26,
    StringMap = 27,
    UuidMap = 28,
    U8Set = 29,
    I8Set = 30,
    U16Set = 31,
    I16Set = 32,
    U32Set = 33,
    I32Set = 34,
    U64Set = 35,
    I64Set = 36,
    StringSet = 37,
    UuidSet = 38,
    Struct = 39,
    Enum = 40,
    Sender = 41,
    Receiver = 42,
}

/// Wrapper for `Vec<u8>` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    pub fn new<T: Into<Vec<u8>>>(bytes: T) -> Self {
        Self(bytes.into())
    }
}

impl Deref for Bytes {
    type Target = ByteSlice;

    fn deref(&self) -> &ByteSlice {
        ByteSlice::new(&self.0)
    }
}

impl AsRef<ByteSlice> for Bytes {
    fn as_ref(&self) -> &ByteSlice {
        self
    }
}

impl Borrow<ByteSlice> for Bytes {
    fn borrow(&self) -> &ByteSlice {
        self
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

impl PartialEq<ByteSlice> for Bytes {
    fn eq(&self, other: &ByteSlice) -> bool {
        **self == *other
    }
}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *self == ***other
    }
}

impl Serialize for Bytes {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl Deserialize for Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_to_vec().map(Bytes)
    }
}

impl AsSerializeArg for Bytes {
    type SerializeArg<'a> = &'a ByteSlice;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for Bytes {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

/// Wrapper for `[u8]` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ByteSlice(pub [u8]);

impl ByteSlice {
    pub fn new<T: AsRef<[u8]> + ?Sized>(bytes: &T) -> &Self {
        let self_ptr = bytes.as_ref() as *const [u8] as *const Self;
        // Safe because of repr(transparent).
        unsafe { &*self_ptr }
    }
}

impl Deref for ByteSlice {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for ByteSlice {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl AsRef<ByteSlice> for [u8] {
    fn as_ref(&self) -> &ByteSlice {
        ByteSlice::new(self)
    }
}

impl<'a, T: AsRef<[u8]>> From<&'a T> for &'a ByteSlice {
    fn from(bytes: &'a T) -> Self {
        ByteSlice::new(bytes)
    }
}

impl ToOwned for ByteSlice {
    type Owned = Bytes;

    fn to_owned(&self) -> Bytes {
        Bytes::new(&self.0)
    }
}

impl PartialEq<Bytes> for ByteSlice {
    fn eq(&self, other: &Bytes) -> bool {
        *self == **other
    }
}

impl PartialEq<[u8]> for ByteSlice {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl PartialEq<ByteSlice> for [u8] {
    fn eq(&self, other: &ByteSlice) -> bool {
        *self == **other
    }
}

impl Serialize for ByteSlice {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl AsSerializeArg for ByteSlice {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ByteSlice {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

/// Empty value that deserializes from everything by skipping over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skip;

impl Deserialize for Skip {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.skip().map(|_| Self)
    }
}

impl<T: Serialize + ?Sized> Serialize for &T {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl<T: AsSerializeArg + ?Sized> AsSerializeArg for &T {
    type SerializeArg<'a>
        = T::SerializeArg<'a>
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        (**self).as_serialize_arg()
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

impl<T: Serialize + ?Sized> Serialize for &mut T {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl<T: AsSerializeArg + ?Sized> AsSerializeArg for &mut T {
    type SerializeArg<'a>
        = T::SerializeArg<'a>
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        (**self).as_serialize_arg()
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

impl<T: Serialize + ?Sized> Serialize for Box<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        T::deserialize(deserializer).map(Self::new)
    }
}

impl<T: AsSerializeArg + ?Sized> AsSerializeArg for Box<T> {
    type SerializeArg<'a>
        = T::SerializeArg<'a>
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        (**self).as_serialize_arg()
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

impl Serialize for () {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_none();
        Ok(())
    }
}

impl Deserialize for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_none()
    }
}

impl AsSerializeArg for () {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
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

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Some(value) => serializer.serialize_some(value)?,
            None => serializer.serialize_none(),
        }

        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_option()
    }
}

impl<T: AsSerializeArg> AsSerializeArg for Option<T> {
    type SerializeArg<'a>
        = Option<T::SerializeArg<'a>>
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self.as_ref().map(AsSerializeArg::as_serialize_arg)
    }
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

impl Serialize for bool {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bool(*self);
        Ok(())
    }
}

impl Deserialize for bool {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bool()
    }
}

impl AsSerializeArg for bool {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
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

impl Serialize for u8 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_u8(*self);
        Ok(())
    }
}

impl Deserialize for u8 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u8()
    }
}

impl AsSerializeArg for u8 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for i8 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_i8(*self);
        Ok(())
    }
}

impl Deserialize for i8 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i8()
    }
}

impl AsSerializeArg for i8 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for u16 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_u16(*self);
        Ok(())
    }
}

impl Deserialize for u16 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u16()
    }
}

impl AsSerializeArg for u16 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for i16 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_i16(*self);
        Ok(())
    }
}

impl Deserialize for i16 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i16()
    }
}

impl AsSerializeArg for i16 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for u32 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_u32(*self);
        Ok(())
    }
}

impl Deserialize for u32 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u32()
    }
}

impl AsSerializeArg for u32 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for i32 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_i32(*self);
        Ok(())
    }
}

impl Deserialize for i32 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i32()
    }
}

impl AsSerializeArg for i32 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for u64 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_u64(*self);
        Ok(())
    }
}

impl Deserialize for u64 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u64()
    }
}

impl AsSerializeArg for u64 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for i64 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_i64(*self);
        Ok(())
    }
}

impl Deserialize for i64 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i64()
    }
}

impl AsSerializeArg for i64 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for f32 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_f32(*self);
        Ok(())
    }
}

impl Deserialize for f32 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_f32()
    }
}

impl AsSerializeArg for f32 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for f64 {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_f64(*self);
        Ok(())
    }
}

impl Deserialize for f64 {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_f64()
    }
}

impl AsSerializeArg for f64 {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl Serialize for str {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

impl AsSerializeArg for str {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
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

impl Serialize for String {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

impl Deserialize for String {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_string()
    }
}

impl AsSerializeArg for String {
    type SerializeArg<'a> = &'a str;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
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

impl Serialize for Uuid {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(*self);
        Ok(())
    }
}

impl Deserialize for Uuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid()
    }
}

impl AsSerializeArg for Uuid {
    type SerializeArg<'a> = Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
    }
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

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec_extend_new()
    }
}

impl<T: Serialize> AsSerializeArg for Vec<T> {
    type SerializeArg<'a>
        = &'a [T]
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

impl<T: Serialize> Serialize for VecDeque<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Deserialize> Deserialize for VecDeque<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec_extend_new()
    }
}

impl<T: Serialize> AsSerializeArg for VecDeque<T> {
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

impl<T: Serialize> Serialize for LinkedList<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Deserialize> Deserialize for LinkedList<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec_extend_new()
    }
}

impl<T: Serialize> AsSerializeArg for LinkedList<T> {
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

impl<T: Serialize> Serialize for [T] {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Serialize> AsSerializeArg for [T] {
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

impl<T: Serialize, const N: usize> Serialize for [T; N] {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Deserialize, const N: usize> Deserialize for [T; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_vec()?;

        if deserializer.len() != N {
            return Err(DeserializeError::UnexpectedValue);
        }

        // SAFETY: This create an array of MaybeUninit<T>, which don't require initialization.
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Manually count number of elements, so that the safety of this function doesn't depend on
        // the correctness of VecDeserializer.
        let mut num = 0;

        for elem in &mut arr {
            match deserializer.deserialize_element() {
                Ok(value) => {
                    elem.write(value);
                    num += 1;
                }

                Err(e) => {
                    for elem in &mut arr[..num] {
                        // SAFETY: The first num elements have been initialized.
                        unsafe {
                            elem.assume_init_drop();
                        }
                    }

                    return Err(e);
                }
            }
        }

        // Panic, because this would indicate a bug in this crate.
        assert_eq!(num, N);

        // SAFETY: Exactly num elements have been and num equals N.
        //
        // It's currently impossible to transmute [MaybeUninit<T>; N] to [T; N] when T is a generic
        // or N a const generic. See https://github.com/rust-lang/rust/issues/61956.
        let value = unsafe {
            (*(&MaybeUninit::new(arr) as *const _ as *const MaybeUninit<[T; N]>)).assume_init_read()
        };

        deserializer.finish(value)
    }
}

impl<T: Serialize, const N: usize> AsSerializeArg for [T; N] {
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
