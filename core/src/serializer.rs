mod bytes;
mod map;
mod set;
mod struct_;
mod vec;

use crate::buf_ext::BufMutExt;
use crate::tags::{self, KeyTag, Tag};
use crate::{
    AsUnknownFields, AsUnknownVariant, ChannelCookie, MAX_VALUE_DEPTH, ObjectId, Serialize,
    SerializeError, SerializeKey, SerializedValueSlice, ServiceId, ValueKind,
};
use ::bytes::{BufMut, BytesMut};
use uuid::Uuid;

pub use self::bytes::{Bytes1Serializer, Bytes2Serializer};
pub use map::{Map1Serializer, Map2Serializer};
pub use set::{Set1Serializer, Set2Serializer};
pub use struct_::{Struct1Serializer, Struct2Serializer};
pub use vec::{Vec1Serializer, Vec2Serializer};

#[derive(Debug)]
pub struct Serializer<'a> {
    buf: &'a mut BytesMut,
    depth: u8,
}

impl<'a> Serializer<'a> {
    pub(crate) fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        let mut this = Self { buf, depth };
        this.increment_depth()?;
        Ok(this)
    }

    fn increment_depth(&mut self) -> Result<(), SerializeError> {
        self.depth += 1;

        if self.depth <= MAX_VALUE_DEPTH {
            Ok(())
        } else {
            Err(SerializeError::TooDeeplyNested)
        }
    }

    pub fn copy_from_serialized_value(
        self,
        value: &SerializedValueSlice,
    ) -> Result<(), SerializeError> {
        self.buf.extend_from_slice(value);
        Ok(())
    }

    pub fn serialize<T: Tag>(self, value: impl Serialize<T>) -> Result<(), SerializeError> {
        value.serialize(self)
    }

    pub fn serialize_none(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }

    pub fn serialize_some<T: Tag>(
        mut self,
        value: impl Serialize<T>,
    ) -> Result<(), SerializeError> {
        self.increment_depth()?;
        self.buf.put_discriminant_u8(ValueKind::Some);
        self.serialize(value)
    }

    pub fn serialize_bool(self, value: bool) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Bool);
        self.buf.put_u8(value.into());
        Ok(())
    }

    pub fn serialize_u8(self, value: u8) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::U8);
        self.buf.put_u8(value);
        Ok(())
    }

    pub fn serialize_i8(self, value: i8) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::I8);
        self.buf.put_i8(value);
        Ok(())
    }

    pub fn serialize_u16(self, value: u16) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::U16);
        self.buf.put_varint_u16_le(value);
        Ok(())
    }

    pub fn serialize_i16(self, value: i16) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::I16);
        self.buf.put_varint_i16_le(value);
        Ok(())
    }

    pub fn serialize_u32(self, value: u32) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::U32);
        self.buf.put_varint_u32_le(value);
        Ok(())
    }

    pub fn serialize_i32(self, value: i32) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::I32);
        self.buf.put_varint_i32_le(value);
        Ok(())
    }

    pub fn serialize_u64(self, value: u64) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::U64);
        self.buf.put_varint_u64_le(value);
        Ok(())
    }

    pub fn serialize_i64(self, value: i64) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::I64);
        self.buf.put_varint_i64_le(value);
        Ok(())
    }

    pub fn serialize_f32(self, value: f32) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::F32);
        self.buf.put_u32_le(value.to_bits());
        Ok(())
    }

    pub fn serialize_f64(self, value: f64) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::F64);
        self.buf.put_u64_le(value.to_bits());
        Ok(())
    }

    pub fn serialize_string(self, value: &str) -> Result<(), SerializeError> {
        if value.len() <= u32::MAX as usize {
            self.buf.put_discriminant_u8(ValueKind::String);
            self.buf.put_varint_u32_le(value.len() as u32);
            self.buf.put_slice(value.as_bytes());
            Ok(())
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn serialize_uuid(self, value: Uuid) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Uuid);
        self.buf.put_slice(value.as_bytes());
        Ok(())
    }

    pub fn serialize_object_id(self, value: ObjectId) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::ObjectId);
        self.buf.put_slice(value.uuid.0.as_bytes());
        self.buf.put_slice(value.cookie.0.as_bytes());
        Ok(())
    }

    pub fn serialize_service_id(self, value: ServiceId) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::ServiceId);
        self.buf.put_slice(value.object_id.uuid.0.as_bytes());
        self.buf.put_slice(value.object_id.cookie.0.as_bytes());
        self.buf.put_slice(value.uuid.0.as_bytes());
        self.buf.put_slice(value.cookie.0.as_bytes());
        Ok(())
    }

    pub fn serialize_vec1(self, num_elems: usize) -> Result<Vec1Serializer<'a>, SerializeError> {
        Vec1Serializer::new(self.buf, num_elems, self.depth)
    }

    pub fn serialize_vec1_iter<T, U>(self, vec: U) -> Result<(), SerializeError>
    where
        T: Tag,
        U: IntoIterator,
        U::IntoIter: ExactSizeIterator,
        U::Item: Serialize<T>,
    {
        let vec = vec.into_iter();
        let mut serializer = self.serialize_vec1(vec.len())?;

        for elem in vec {
            serializer.serialize(elem)?;
        }

        serializer.finish()
    }

    pub fn serialize_vec2(self) -> Result<Vec2Serializer<'a>, SerializeError> {
        Vec2Serializer::new(self.buf, self.depth)
    }

    pub fn serialize_vec2_iter<T, U>(self, vec: U) -> Result<(), SerializeError>
    where
        T: Tag,
        U: IntoIterator,
        U::Item: Serialize<T>,
    {
        let mut serializer = self.serialize_vec2()?;

        for elem in vec {
            serializer.serialize(elem)?;
        }

        serializer.finish()
    }

    pub fn serialize_bytes1(
        self,
        num_elems: usize,
    ) -> Result<Bytes1Serializer<'a>, SerializeError> {
        Bytes1Serializer::new(self.buf, num_elems)
    }

    pub fn serialize_byte_slice1(self, bytes: &[u8]) -> Result<(), SerializeError> {
        let mut serializer = self.serialize_bytes1(bytes.len())?;
        serializer.serialize(bytes)?;
        serializer.finish()
    }

    pub fn serialize_bytes2(self) -> Result<Bytes2Serializer<'a>, SerializeError> {
        Bytes2Serializer::new(self.buf)
    }

    pub fn serialize_byte_slice2(self, bytes: &[u8]) -> Result<(), SerializeError> {
        let mut serializer = self.serialize_bytes2()?;
        serializer.serialize(bytes)?;
        serializer.finish()
    }

    pub fn serialize_map1<K: KeyTag>(
        self,
        num_elems: usize,
    ) -> Result<Map1Serializer<'a, K>, SerializeError> {
        Map1Serializer::new(self.buf, num_elems, self.depth)
    }

    pub fn serialize_map1_iter<K, L, T, U, I>(self, map: I) -> Result<(), SerializeError>
    where
        K: KeyTag,
        L: SerializeKey<K>,
        T: Tag,
        U: Serialize<T>,
        I: IntoIterator<Item = (L, U)>,
        I::IntoIter: ExactSizeIterator,
    {
        let map = map.into_iter();
        let mut serializer = self.serialize_map1(map.len())?;

        for (key, value) in map {
            serializer.serialize(&key, value)?;
        }

        serializer.finish()
    }

    pub fn serialize_map2<K: KeyTag>(self) -> Result<Map2Serializer<'a, K>, SerializeError> {
        Map2Serializer::new(self.buf, self.depth)
    }

    pub fn serialize_map2_iter<K, L, T, U, I>(self, map: I) -> Result<(), SerializeError>
    where
        K: KeyTag,
        L: SerializeKey<K>,
        T: Tag,
        U: Serialize<T>,
        I: IntoIterator<Item = (L, U)>,
    {
        let mut serializer = self.serialize_map2()?;

        for (key, value) in map {
            serializer.serialize(&key, value)?;
        }

        serializer.finish()
    }

    pub fn serialize_set1<K: KeyTag>(
        self,
        num_elems: usize,
    ) -> Result<Set1Serializer<'a, K>, SerializeError> {
        Set1Serializer::new(self.buf, num_elems)
    }

    pub fn serialize_set1_iter<K, T>(self, set: T) -> Result<(), SerializeError>
    where
        K: KeyTag,
        T: IntoIterator,
        T::IntoIter: ExactSizeIterator,
        T::Item: SerializeKey<K>,
    {
        let set = set.into_iter();
        let mut serializer = self.serialize_set1(set.len())?;

        for value in set {
            serializer.serialize(&value)?;
        }

        serializer.finish()
    }

    pub fn serialize_set2<K: KeyTag>(self) -> Result<Set2Serializer<'a, K>, SerializeError> {
        Set2Serializer::new(self.buf)
    }

    pub fn serialize_set2_iter<K, T>(self, set: T) -> Result<(), SerializeError>
    where
        K: KeyTag,
        T: IntoIterator,
        T::Item: SerializeKey<K>,
    {
        let mut serializer = self.serialize_set2()?;

        for value in set {
            serializer.serialize(&value)?;
        }

        serializer.finish()
    }

    pub fn serialize_struct1(
        self,
        num_fields: usize,
    ) -> Result<Struct1Serializer<'a>, SerializeError> {
        Struct1Serializer::new(self.buf, num_fields, self.depth)
    }

    pub fn serialize_struct1_with_unknown_fields<T>(
        self,
        num_fields: usize,
        unknown_fields: T,
    ) -> Result<Struct1Serializer<'a>, SerializeError>
    where
        T: AsUnknownFields,
        T::FieldsIter: ExactSizeIterator,
    {
        Struct1Serializer::with_unknown_fields(self.buf, num_fields, unknown_fields, self.depth)
    }

    pub fn serialize_struct2(self) -> Result<Struct2Serializer<'a>, SerializeError> {
        Struct2Serializer::new(self.buf, self.depth)
    }

    pub fn serialize_struct2_with_unknown_fields(
        self,
        unknown_fields: impl AsUnknownFields,
    ) -> Result<Struct2Serializer<'a>, SerializeError> {
        Struct2Serializer::with_unknown_fields(self.buf, unknown_fields, self.depth)
    }

    pub fn serialize_enum<T: Tag>(
        mut self,
        id: impl Into<u32>,
        value: impl Serialize<T>,
    ) -> Result<(), SerializeError> {
        self.increment_depth()?;
        self.buf.put_discriminant_u8(ValueKind::Enum);
        self.buf.put_varint_u32_le(id.into());
        self.serialize(value)
    }

    pub fn serialize_unit_enum(self, id: impl Into<u32>) -> Result<(), SerializeError> {
        self.serialize_enum::<tags::Unit>(id, ())
    }

    pub fn serialize_unknown_variant(
        self,
        variant: impl AsUnknownVariant,
    ) -> Result<(), SerializeError> {
        self.serialize_enum(variant.id(), variant.value())
    }

    pub fn serialize_sender(self, value: ChannelCookie) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Sender);
        self.buf.put_slice(value.0.as_bytes());
        Ok(())
    }

    pub fn serialize_receiver(self, value: ChannelCookie) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Receiver);
        self.buf.put_slice(value.0.as_bytes());
        Ok(())
    }
}
