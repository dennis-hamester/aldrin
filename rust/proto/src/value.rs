#[cfg(test)]
mod test;

use crate::deserialize_key::DeserializeKey;
use crate::error::{DeserializeError, SerializeError};
use crate::serialize_key::SerializeKey;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::mem::MaybeUninit;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedValue {
    buf: BytesMut,
}

impl SerializedValue {
    /// Cheaply creates an empty `SerializedValue`.
    pub fn empty() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    pub fn serialize<T: Serialize + ?Sized>(value: &T) -> Result<Self, SerializeError> {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length.
        let mut buf = BytesMut::zeroed(9);
        let serializer = Serializer::new(&mut buf);
        value.serialize(serializer)?;
        Ok(Self { buf })
    }

    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length.
        let mut buf = &self.buf[9..];
        let deserializer = Deserializer::new(&mut buf);

        let res = T::deserialize(deserializer);

        if res.is_ok() && !buf.is_empty() {
            return Err(DeserializeError::TrailingData);
        }

        res
    }

    pub(crate) fn from_bytes_mut(buf: BytesMut) -> Self {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        debug_assert!(buf.len() >= 10);

        Self { buf }
    }

    pub(crate) fn into_bytes_mut(self) -> BytesMut {
        self.buf
    }
}

/// Wrapper for `Vec<u8>` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

/// Wrapper for `&[u8]` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytesRef<'a>(pub &'a [u8]);

impl<T: Serialize + ?Sized> Serialize for &T {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (*self).serialize(serializer)
    }
}

impl<T: Serialize + ?Sized> Serialize for Box<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        T::deserialize(deserializer).map(Box::new)
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

impl Serialize for str {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
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

impl<T: Serialize> Serialize for [T] {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
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

        if deserializer.remaining_elements() != N {
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
        // It's impossible to transmute [MaybeUninit<T>; N] to [T; N] on 1.64.0 when T is a generic
        // or N a const generic. See https://github.com/rust-lang/rust/issues/61956.
        let value = unsafe {
            (*(&MaybeUninit::new(arr) as *const _ as *const MaybeUninit<[T; N]>)).assume_init_read()
        };

        Ok(value)
    }
}

impl Serialize for Bytes {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(&self.0)
    }
}

impl Deserialize for Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_to_vec().map(Bytes)
    }
}

impl<'a> Serialize for BytesRef<'a> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self.0)
    }
}

impl Serialize for bytes::Bytes {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize for bytes::Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize_bytes_to_vec()
            .map(bytes::Bytes::from)
    }
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
        Ok(bytes::BytesMut::from(vec.as_slice()))
    }
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
