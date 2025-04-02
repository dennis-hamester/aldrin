use crate::buf_ext::BufMutExt;
use crate::tags::{self, KeyTag, KeyTagImpl, Tag};
use crate::{
    AsUnknownFields, AsUnknownVariant, ChannelCookie, ObjectId, Serialize, SerializeError,
    SerializeKey, SerializedValueSlice, ServiceId, UnknownFieldsRef, ValueKind, MAX_VALUE_DEPTH,
};
use bytes::{BufMut, BytesMut};
use std::fmt;
use std::marker::PhantomData;
use uuid::Uuid;

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

    pub fn serialize<T: Tag, U: Serialize<T>>(self, value: U) -> Result<(), SerializeError> {
        value.serialize(self)
    }

    pub fn serialize_none(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }

    pub fn serialize_some<T: Tag, U: Serialize<T>>(
        mut self,
        value: U,
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

    pub fn serialize_bytes(self, num_elems: usize) -> Result<BytesSerializer<'a>, SerializeError> {
        BytesSerializer::new(self.buf, num_elems)
    }

    pub fn serialize_byte_slice(self, bytes: &[u8]) -> Result<(), SerializeError> {
        let mut serializer = self.serialize_bytes(bytes.len())?;
        serializer.serialize(bytes)?;
        serializer.finish()
    }

    pub fn serialize_map<K: KeyTag>(
        self,
        num_elems: usize,
    ) -> Result<MapSerializer<'a, K>, SerializeError> {
        MapSerializer::new(self.buf, num_elems, self.depth)
    }

    pub fn serialize_map_iter<K, L, T, U, I>(self, map: I) -> Result<(), SerializeError>
    where
        K: KeyTag,
        L: SerializeKey<K>,
        T: Tag,
        U: Serialize<T>,
        I: IntoIterator<Item = (L, U)>,
        I::IntoIter: ExactSizeIterator,
    {
        let map = map.into_iter();
        let mut serializer = self.serialize_map(map.len())?;

        for (key, value) in map {
            serializer.serialize(&key, value)?;
        }

        serializer.finish()
    }

    pub fn serialize_set<K: KeyTag>(
        self,
        num_elems: usize,
    ) -> Result<SetSerializer<'a, K>, SerializeError> {
        SetSerializer::new(self.buf, num_elems)
    }

    pub fn serialize_set_iter<K, T>(self, set: T) -> Result<(), SerializeError>
    where
        K: KeyTag,
        T: IntoIterator,
        T::IntoIter: ExactSizeIterator,
        T::Item: SerializeKey<K>,
    {
        let set = set.into_iter();
        let mut serializer = self.serialize_set(set.len())?;

        for value in set {
            serializer.serialize(&value)?;
        }

        serializer.finish()
    }

    pub fn serialize_struct(
        self,
        num_fields: usize,
    ) -> Result<StructSerializer<'a>, SerializeError> {
        StructSerializer::new(self.buf, num_fields, self.depth)
    }

    pub fn serialize_struct_with_unknown_fields(
        self,
        num_fields: usize,
        unknown_fields: impl AsUnknownFields,
    ) -> Result<StructSerializer<'a>, SerializeError> {
        StructSerializer::with_unknown_fields(self.buf, num_fields, unknown_fields, self.depth)
    }

    pub fn serialize_enum<T: Tag, U: Serialize<T>>(
        mut self,
        variant: impl Into<u32>,
        value: U,
    ) -> Result<(), SerializeError> {
        self.increment_depth()?;
        self.buf.put_discriminant_u8(ValueKind::Enum);
        self.buf.put_varint_u32_le(variant.into());
        self.serialize(value)
    }

    pub fn serialize_unit_enum(self, variant: impl Into<u32>) -> Result<(), SerializeError> {
        self.serialize_enum::<tags::Unit, _>(variant, ())
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

#[derive(Debug)]
pub struct Vec1Serializer<'a> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    depth: u8,
}

impl<'a> Vec1Serializer<'a> {
    fn new(buf: &'a mut BytesMut, num_elems: usize, depth: u8) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Vec1);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
                depth,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

#[derive(Debug)]
pub struct Vec2Serializer<'a> {
    buf: &'a mut BytesMut,
    depth: u8,
}

impl<'a> Vec2Serializer<'a> {
    fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        buf.put_discriminant_u8(ValueKind::Vec2);
        Ok(Self { buf, depth })
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Some);

        let serializer = Serializer::new(self.buf, self.depth)?;
        serializer.serialize(value)?;

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }
}

#[derive(Debug)]
pub struct BytesSerializer<'a> {
    buf: &'a mut BytesMut,
    num_elems: u32,
}

impl<'a> BytesSerializer<'a> {
    fn new(buf: &'a mut BytesMut, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Bytes);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize(&mut self, bytes: &[u8]) -> Result<&mut Self, SerializeError> {
        if bytes.len() <= self.num_elems as usize {
            self.num_elems -= bytes.len() as u32;
            self.buf.put_slice(bytes);
            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

pub struct MapSerializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> MapSerializer<'a, K> {
    fn new(mut buf: &'a mut BytesMut, num_elems: usize, depth: u8) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            K::Impl::serialize_map_value_kind(&mut buf);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
                depth,
                _key: PhantomData,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize<L: SerializeKey<K> + ?Sized, T: Tag, U: Serialize<T>>(
        &mut self,
        key: &L,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;

            K::Impl::serialize_key(key.try_as_key()?, self.buf)?;

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

impl<K> fmt::Debug for MapSerializer<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);
        f.field("depth", &self.depth);

        f.finish()
    }
}

pub struct SetSerializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> SetSerializer<'a, K> {
    fn new(mut buf: &'a mut BytesMut, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            K::Impl::serialize_set_value_kind(&mut buf);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
                _key: PhantomData,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize<T: SerializeKey<K> + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            K::Impl::serialize_key(value.try_as_key()?, self.buf)?;
            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

impl<T> fmt::Debug for SetSerializer<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SetSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);

        f.finish()
    }
}

#[derive(Debug)]
pub struct StructSerializer<'a> {
    buf: &'a mut BytesMut,
    num_fields: u32,
    depth: u8,
}

impl<'a> StructSerializer<'a> {
    fn new(buf: &'a mut BytesMut, num_fields: usize, depth: u8) -> Result<Self, SerializeError> {
        if num_fields <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Struct);
            buf.put_varint_u32_le(num_fields as u32);

            Ok(Self {
                buf,
                num_fields: num_fields as u32,
                depth,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    fn with_unknown_fields(
        buf: &'a mut BytesMut,
        num_fields: usize,
        unknown_fields: impl AsUnknownFields,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        let unknown_fields = unknown_fields.fields();
        let mut this = Self::new(buf, num_fields + unknown_fields.len(), depth)?;
        this.serialize_unknown_fields(UnknownFieldsRef(unknown_fields))?;
        Ok(this)
    }

    pub fn remaining_fields(&self) -> usize {
        self.num_fields as usize
    }

    pub fn requires_additional_fields(&self) -> bool {
        self.num_fields > 0
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        id: impl Into<u32>,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_fields > 0 {
            self.num_fields -= 1;

            self.buf.put_varint_u32_le(id.into());

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn serialize_unknown_fields(
        &mut self,
        unknown_fields: impl AsUnknownFields,
    ) -> Result<&mut Self, SerializeError> {
        for (id, value) in unknown_fields.fields() {
            if self.num_fields == 0 {
                return Err(SerializeError::TooManyElements);
            }

            self.num_fields -= 1;
            self.buf.put_varint_u32_le(id);

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;
        }

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_fields == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}
