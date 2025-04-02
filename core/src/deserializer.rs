use crate::buf_ext::ValueBufExt;
use crate::tags::{self, KeyTag, KeyTagImpl, Tag};
use crate::{
    ChannelCookie, Deserialize, DeserializeError, DeserializeKey, ObjectCookie, ObjectId,
    ObjectUuid, SerializedValueSlice, ServiceCookie, ServiceId, ServiceUuid, UnknownFields,
    UnknownVariant, ValueKind, MAX_VALUE_DEPTH,
};
use bytes::Buf;
use std::marker::PhantomData;
use std::{fmt, iter};
use uuid::Uuid;

#[derive(Debug)]
pub struct Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    depth: u8,
}

impl<'a, 'b> Deserializer<'a, 'b> {
    pub(crate) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let mut this = Self { buf, depth };
        this.increment_depth()?;
        Ok(this)
    }

    fn increment_depth(&mut self) -> Result<(), DeserializeError> {
        self.depth += 1;
        if self.depth <= MAX_VALUE_DEPTH {
            Ok(())
        } else {
            Err(DeserializeError::TooDeeplyNested)
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<usize, DeserializeError> {
        let mut buf = *self.buf;
        Deserializer::new(&mut buf, self.depth - 1)?.skip()?;

        // Determine the length by computing how far `skip()` has advanced `buf` compared to the
        // original buffer `*self.buf`.
        Ok(buf.as_ptr() as usize - (*self.buf).as_ptr() as usize)
    }

    pub fn split_off_serialized_value(self) -> Result<&'b SerializedValueSlice, DeserializeError> {
        let len = self.len()?;
        let res = SerializedValueSlice::new(&self.buf[..len]);
        self.buf.advance(len);
        Ok(res)
    }

    pub fn peek_value_kind(&self) -> Result<ValueKind, DeserializeError> {
        self.buf.try_peek_discriminant_u8()
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::None => Ok(()),

            ValueKind::Some => {
                self.increment_depth()?;
                self.skip()
            }

            ValueKind::Bool | ValueKind::U8 | ValueKind::I8 => self.buf.try_skip(1),
            ValueKind::U16 | ValueKind::I16 => self.buf.try_skip_varint_le::<2>(),
            ValueKind::U32 | ValueKind::I32 => self.buf.try_skip_varint_le::<4>(),
            ValueKind::U64 | ValueKind::I64 => self.buf.try_skip_varint_le::<8>(),
            ValueKind::F32 => self.buf.try_skip(4),
            ValueKind::F64 => self.buf.try_skip(8),

            ValueKind::String => {
                let len = self.buf.try_get_varint_u32_le()? as usize;
                self.buf.try_skip(len)
            }

            ValueKind::Uuid | ValueKind::Sender | ValueKind::Receiver => self.buf.try_skip(16),
            ValueKind::ObjectId => self.buf.try_skip(32),
            ValueKind::ServiceId => self.buf.try_skip(64),

            ValueKind::Vec1 => {
                Vec1Deserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Bytes => BytesDeserializer::new_without_value_kind(self.buf)?.skip(),

            ValueKind::U8Map => {
                MapDeserializer::<tags::U8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I8Map => {
                MapDeserializer::<tags::I8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U16Map => {
                MapDeserializer::<tags::U16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I16Map => {
                MapDeserializer::<tags::I16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U32Map => {
                MapDeserializer::<tags::U32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I32Map => {
                MapDeserializer::<tags::I32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U64Map => {
                MapDeserializer::<tags::U64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I64Map => {
                MapDeserializer::<tags::I64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::StringMap => {
                MapDeserializer::<tags::String>::new_without_value_kind(self.buf, self.depth)?
                    .skip()
            }

            ValueKind::UuidMap => {
                MapDeserializer::<tags::Uuid>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U8Set => {
                SetDeserializer::<tags::U8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I8Set => {
                SetDeserializer::<tags::I8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U16Set => {
                SetDeserializer::<tags::U16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I16Set => {
                SetDeserializer::<tags::I16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U32Set => {
                SetDeserializer::<tags::U32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I32Set => {
                SetDeserializer::<tags::I32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U64Set => {
                SetDeserializer::<tags::U64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I64Set => {
                SetDeserializer::<tags::I64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::StringSet => {
                SetDeserializer::<tags::String>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::UuidSet => {
                SetDeserializer::<tags::Uuid>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::Struct => {
                StructDeserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Enum => {
                EnumDeserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Vec2 => {
                Vec2Deserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }
        }
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<U, DeserializeError> {
        U::deserialize(self)
    }

    pub fn deserialize_none(self) -> Result<(), DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::None)
    }

    pub fn deserialize_some<T: Tag, U: Deserialize<T>>(mut self) -> Result<U, DeserializeError> {
        self.increment_depth()?;
        self.buf.ensure_discriminant_u8(ValueKind::Some)?;
        self.deserialize()
    }

    pub fn deserialize_option<T: Tag, U: Deserialize<T>>(
        mut self,
    ) -> Result<Option<U>, DeserializeError> {
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::Some => {
                self.increment_depth()?;
                self.deserialize().map(Some)
            }

            ValueKind::None => Ok(None),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }

    pub fn deserialize_bool(self) -> Result<bool, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Bool)?;

        self.buf
            .try_get_u8()
            .map(|v| v != 0)
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    pub fn deserialize_u8(self) -> Result<u8, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::U8)?;

        self.buf
            .try_get_u8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    pub fn deserialize_i8(self) -> Result<i8, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::I8)?;

        self.buf
            .try_get_i8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    pub fn deserialize_u16(self) -> Result<u16, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::U16)?;
        self.buf.try_get_varint_u16_le()
    }

    pub fn deserialize_i16(self) -> Result<i16, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::I16)?;
        self.buf.try_get_varint_i16_le()
    }

    pub fn deserialize_u32(self) -> Result<u32, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::U32)?;
        self.buf.try_get_varint_u32_le()
    }

    pub fn deserialize_i32(self) -> Result<i32, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::I32)?;
        self.buf.try_get_varint_i32_le()
    }

    pub fn deserialize_u64(self) -> Result<u64, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::U64)?;
        self.buf.try_get_varint_u64_le()
    }

    pub fn deserialize_i64(self) -> Result<i64, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::I64)?;
        self.buf.try_get_varint_i64_le()
    }

    pub fn deserialize_f32(self) -> Result<f32, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::F32)?;

        self.buf
            .try_get_f32_le()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    pub fn deserialize_f64(self) -> Result<f64, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::F64)?;

        self.buf
            .try_get_f64_le()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    pub fn deserialize_string(self) -> Result<String, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::String)?;
        let len = self.buf.try_get_varint_u32_le()? as usize;
        let bytes = self.buf.try_copy_to_bytes(len)?.into();
        String::from_utf8(bytes).map_err(|_| DeserializeError::InvalidSerialization)
    }

    pub fn deserialize_uuid(self) -> Result<Uuid, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Uuid)?;
        let mut bytes = uuid::Bytes::default();

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        Ok(Uuid::from_bytes(bytes))
    }

    pub fn deserialize_object_id(self) -> Result<ObjectId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ObjectId)?;
        let mut bytes = uuid::Bytes::default();

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let uuid = ObjectUuid(Uuid::from_bytes(bytes));

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let cookie = ObjectCookie(Uuid::from_bytes(bytes));

        Ok(ObjectId::new(uuid, cookie))
    }

    pub fn deserialize_service_id(self) -> Result<ServiceId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ServiceId)?;
        let mut bytes = uuid::Bytes::default();

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let object_uuid = ObjectUuid(Uuid::from_bytes(bytes));

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let object_cookie = ObjectCookie(Uuid::from_bytes(bytes));

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let service_uuid = ServiceUuid(Uuid::from_bytes(bytes));

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        let service_cookie = ServiceCookie(Uuid::from_bytes(bytes));

        Ok(ServiceId::new(
            ObjectId::new(object_uuid, object_cookie),
            service_uuid,
            service_cookie,
        ))
    }

    pub fn deserialize_vec(self) -> Result<VecDeserializer<'a, 'b>, DeserializeError> {
        VecDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_vec_extend<T, U, V>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        self.deserialize_vec()?.deserialize_extend(vec)
    }

    pub fn deserialize_vec_extend_new<T, U, V>(self) -> Result<V, DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<U>,
    {
        let mut vec = V::default();
        self.deserialize_vec_extend(&mut vec)?;
        Ok(vec)
    }

    pub fn deserialize_vec1(self) -> Result<Vec1Deserializer<'a, 'b>, DeserializeError> {
        Vec1Deserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_vec1_extend<T, U, V>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        self.deserialize_vec1()?.deserialize_extend(vec)
    }

    pub fn deserialize_vec1_extend_new<T, U, V>(self) -> Result<V, DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<U>,
    {
        let mut vec = V::default();
        self.deserialize_vec1_extend(&mut vec)?;
        Ok(vec)
    }

    pub fn deserialize_vec2(self) -> Result<Vec2Deserializer<'a, 'b>, DeserializeError> {
        Vec2Deserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_vec2_extend<T, U, V>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        self.deserialize_vec2()?.deserialize_extend(vec)
    }

    pub fn deserialize_vec2_extend_new<T, U, V>(self) -> Result<V, DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<U>,
    {
        let mut vec = V::default();
        self.deserialize_vec2_extend(&mut vec)?;
        Ok(vec)
    }

    pub fn deserialize_bytes(self) -> Result<BytesDeserializer<'a, 'b>, DeserializeError> {
        BytesDeserializer::new(self.buf)
    }

    pub fn deserialize_bytes_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        self.deserialize_bytes()?.deserialize_extend(bytes)
    }

    pub fn deserialize_bytes_extend_new<T>(self) -> Result<T, DeserializeError>
    where
        T: Default + Extend<u8>,
    {
        let mut bytes = T::default();
        self.deserialize_bytes()?.deserialize_extend(&mut bytes)?;
        Ok(bytes)
    }

    pub fn deserialize_map<K: KeyTag>(
        self,
    ) -> Result<MapDeserializer<'a, 'b, K>, DeserializeError> {
        MapDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_map_extend<K, L, T, U, V>(self, map: &mut V) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        MapDeserializer::new(self.buf, self.depth)?.deserialize_extend(map)
    }

    pub fn deserialize_map_extend_new<K, L, T, U, V>(self) -> Result<V, DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<(L, U)>,
    {
        let mut map = V::default();
        self.deserialize_map_extend(&mut map)?;
        Ok(map)
    }

    pub fn deserialize_set<K: KeyTag>(
        self,
    ) -> Result<SetDeserializer<'a, 'b, K>, DeserializeError> {
        SetDeserializer::new(self.buf)
    }

    pub fn deserialize_set_extend<K, T, U>(self, set: &mut U) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        SetDeserializer::new(self.buf)?.deserialize_extend(set)
    }

    pub fn deserialize_set_extend_new<K, T, U>(self) -> Result<U, DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Default + Extend<T>,
    {
        let mut set = U::default();
        SetDeserializer::new(self.buf)?.deserialize_extend(&mut set)?;
        Ok(set)
    }

    pub fn deserialize_struct(self) -> Result<StructDeserializer<'a, 'b>, DeserializeError> {
        StructDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_enum(self) -> Result<EnumDeserializer<'a, 'b>, DeserializeError> {
        EnumDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_sender(self) -> Result<ChannelCookie, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Sender)?;
        let mut bytes = uuid::Bytes::default();

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }

    pub fn deserialize_receiver(self) -> Result<ChannelCookie, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Receiver)?;
        let mut bytes = uuid::Bytes::default();

        self.buf
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }
}

#[derive(Debug)]
pub enum VecDeserializer<'a, 'b> {
    V1(Vec1Deserializer<'a, 'b>),
    V2(Vec2Deserializer<'a, 'b>),
}

impl<'a, 'b> VecDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        match buf.try_get_discriminant_u8()? {
            ValueKind::Vec1 => Vec1Deserializer::new_without_value_kind(buf, depth).map(Self::V1),
            ValueKind::Vec2 => Vec2Deserializer::new_without_value_kind(buf, depth).map(Self::V2),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(
        &mut self,
    ) -> Result<Option<U>, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.deserialize().map(Some),
            Self::V2(deserializer) => deserializer.deserialize(),
        }
    }

    pub fn deserialize_extend<T, U, V>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_extend(vec),
            Self::V2(deserializer) => deserializer.deserialize_extend(vec),
        }
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.skip_element(),
            Self::V2(deserializer) => deserializer.skip_element(),
        }
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.skip(),
            Self::V2(deserializer) => deserializer.skip(),
        }
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        match self {
            Self::V1(deserializer) => deserializer.finish_with(f),
            Self::V2(deserializer) => deserializer.finish_with(f),
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        match self {
            Self::V1(deserializer) => deserializer.skip_and_finish_with(f),
            Self::V2(deserializer) => deserializer.skip_and_finish_with(f),
        }
    }
}

#[derive(Debug)]
pub struct Vec1Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
}

impl<'a, 'b> Vec1Deserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec1)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, len, depth })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(&mut self) -> Result<U, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            let deserializer = Deserializer::new(self.buf, self.depth)?;
            deserializer.deserialize()
        }
    }

    pub fn deserialize_extend<T, U, V>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        while !self.is_empty() {
            let elem = self.deserialize()?;
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            let deserializer = Deserializer::new(self.buf, self.depth)?;
            deserializer.skip()
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.skip_element()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

#[derive(Debug)]
pub struct Vec2Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    empty: bool,
    depth: u8,
}

impl<'a, 'b> Vec2Deserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec2)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        Ok(Self {
            buf,
            empty: false,
            depth,
        })
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(
        &mut self,
    ) -> Result<Option<U>, DeserializeError> {
        if self.empty {
            Ok(None)
        } else {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => {
                    self.empty = true;
                    Ok(None)
                }

                ValueKind::Some => {
                    let deserializer = Deserializer::new(self.buf, self.depth)?;
                    deserializer.deserialize().map(Some)
                }

                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    pub fn deserialize_extend<T, U, V>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        while let Some(elem) = self.deserialize()? {
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if !self.empty {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => self.empty = true,
                ValueKind::Some => Deserializer::new(self.buf, self.depth)?.skip()?,
                _ => return Err(DeserializeError::InvalidSerialization),
            }
        }

        Ok(())
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.empty {
            self.skip_element()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.empty {
            f()
        } else {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => f(),
                ValueKind::Some => Err(DeserializeError::MoreElementsRemain),
                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

#[derive(Debug)]
pub struct BytesDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
}

impl<'a, 'b> BytesDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Bytes)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        if buf.len() >= len as usize {
            Ok(Self { buf, len })
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &(*self.buf)[..self.len as usize]
    }

    pub fn advance(&mut self, cnt: usize) -> Result<(), DeserializeError> {
        if cnt <= self.len as usize {
            self.buf.try_skip(cnt)?;
            self.len -= cnt as u32;
            Ok(())
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        bytes.extend(self.as_slice().iter().copied());
        self.buf.try_skip(self.len as usize)
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        self.advance(self.len as usize)
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

pub struct MapDeserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> MapDeserializer<'a, 'b, K> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        K::Impl::deserialize_map_value_kind(buf)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            depth,
            _key: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<L: DeserializeKey<K>>(
        &mut self,
    ) -> Result<MapElementDeserializer<'_, 'b, L>, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            MapElementDeserializer::new(self.buf, self.depth)
        }
    }

    pub fn deserialize_element<L, T, U>(&mut self) -> Result<(L, U), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
    {
        self.deserialize()?.deserialize()
    }

    pub fn deserialize_extend<L, T, U, V>(mut self, map: &mut V) -> Result<(), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        while !self.is_empty() {
            let kv = self.deserialize_element()?;
            map.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            K::Impl::skip(self.buf)?;
            Deserializer::new(self.buf, self.depth)?.skip()
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.skip_element()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

impl<K> fmt::Debug for MapDeserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapDeserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);
        f.field("depth", &self.depth);

        f.finish()
    }
}

#[derive(Debug)]
pub struct MapElementDeserializer<'a, 'b, L> {
    buf: &'a mut &'b [u8],
    key: L,
    depth: u8,
}

impl<'a, 'b, L> MapElementDeserializer<'a, 'b, L> {
    fn new<K>(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
    {
        let key = K::Impl::deserialize_key(buf).and_then(L::try_from_key)?;
        Ok(Self { buf, key, depth })
    }

    pub fn key(&self) -> &L {
        &self.key
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<(L, U), DeserializeError> {
        let deserializer = Deserializer::new(self.buf, self.depth)?;
        let value = deserializer.deserialize()?;
        Ok((self.key, value))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }
}

pub struct SetDeserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> SetDeserializer<'a, 'b, K> {
    fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        K::Impl::deserialize_set_value_kind(buf)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            _key: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<T: DeserializeKey<K>>(&mut self) -> Result<T, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            K::Impl::deserialize_key(self.buf).and_then(T::try_from_key)
        }
    }

    pub fn deserialize_extend<T, U>(mut self, set: &mut U) -> Result<(), DeserializeError>
    where
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        while !self.is_empty() {
            let value = self.deserialize()?;
            set.extend(iter::once(value));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            K::Impl::skip(self.buf)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.skip_element()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

impl<K> fmt::Debug for SetDeserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SetDeserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);

        f.finish()
    }
}

#[derive(Debug)]
pub struct StructDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    unknown_fields: UnknownFields,
}

impl<'a, 'b> StructDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Struct)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            depth,
            unknown_fields: UnknownFields::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize(&mut self) -> Result<FieldDeserializer<'_, 'b>, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            FieldDeserializer::new(self.buf, self.depth, &mut self.unknown_fields)
        }
    }

    pub fn deserialize_specific_field<T: Tag, U: Deserialize<T>>(
        &mut self,
        id: impl Into<u32>,
    ) -> Result<U, DeserializeError> {
        let field = self.deserialize()?;

        if field.id() == id.into() {
            field.deserialize()
        } else {
            Err(DeserializeError::InvalidSerialization)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.deserialize()?.skip()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|_| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f(self.unknown_fields)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|_| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(mut self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        while !self.is_empty() {
            self.deserialize()?.skip()?;
        }

        f(self.unknown_fields)
    }
}

#[derive(Debug)]
pub struct FieldDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    id: u32,
    depth: u8,
    unknown_fields: &'a mut UnknownFields,
}

impl<'a, 'b> FieldDeserializer<'a, 'b> {
    fn new(
        buf: &'a mut &'b [u8],
        depth: u8,
        unknown_fields: &'a mut UnknownFields,
    ) -> Result<Self, DeserializeError> {
        let id = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            id,
            depth,
            unknown_fields,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn try_id<T: TryFrom<u32>>(&self) -> Result<T, DeserializeError> {
        self.id
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<U, DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.deserialize()
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }

    pub fn add_to_unknown_fields(self) -> Result<(), DeserializeError> {
        let deserializer = Deserializer::new(self.buf, self.depth)?;
        let value = deserializer.deserialize()?;
        self.unknown_fields.insert(self.id, value);
        Ok(())
    }
}

#[derive(Debug)]
pub struct EnumDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    variant: u32,
    depth: u8,
}

impl<'a, 'b> EnumDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Enum)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let variant = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            variant,
            depth,
        })
    }

    pub fn variant(&self) -> u32 {
        self.variant
    }

    pub fn try_variant<T: TryFrom<u32>>(&self) -> Result<T, DeserializeError> {
        self.variant
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<U, DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.deserialize()
    }

    pub fn deserialize_unit(self) -> Result<(), DeserializeError> {
        self.deserialize::<tags::Unit, _>()
    }

    pub fn into_unknown_variant(self) -> Result<UnknownVariant, DeserializeError> {
        let id = self.variant;
        let value = self.deserialize()?;
        Ok(UnknownVariant::new(id, value))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }
}
