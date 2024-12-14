use crate::buf_ext::ValueBufExt;
use crate::deserialize_key::{DeserializeKey, Sealed as _};
use crate::error::DeserializeError;
use crate::ids::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::unknown_fields::UnknownFields;
use crate::unknown_variant::UnknownVariant;
use crate::value::ValueKind;
use crate::MAX_VALUE_DEPTH;
use bytes::Buf;
use std::iter;
use std::marker::PhantomData;
use uuid::Uuid;

pub trait Deserialize: Sized {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError>;
}

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

    pub fn skip(self) -> Result<(), DeserializeError> {
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::None => Ok(()),
            ValueKind::Some => self.skip(),
            ValueKind::Bool | ValueKind::U8 | ValueKind::I8 => self.buf.try_skip(1),
            ValueKind::U16 => self.buf.try_skip_varint_le::<2>(),
            ValueKind::I16 => self.buf.try_skip_varint_le::<2>(),
            ValueKind::U32 => self.buf.try_skip_varint_le::<4>(),
            ValueKind::I32 => self.buf.try_skip_varint_le::<4>(),
            ValueKind::U64 => self.buf.try_skip_varint_le::<8>(),
            ValueKind::I64 => self.buf.try_skip_varint_le::<8>(),
            ValueKind::F32 => self.buf.try_skip(4),
            ValueKind::F64 => self.buf.try_skip(8),
            ValueKind::String => {
                let len = self.buf.try_get_varint_u32_le()? as usize;
                self.buf.try_skip(len)
            }
            ValueKind::Uuid | ValueKind::Sender | ValueKind::Receiver => self.buf.try_skip(16),
            ValueKind::ObjectId => self.buf.try_skip(32),
            ValueKind::ServiceId => self.buf.try_skip(64),
            ValueKind::Vec => VecDeserializer::new_without_value_kind(self.buf, self.depth)?.skip(),
            ValueKind::Bytes => BytesDeserializer::new_without_value_kind(self.buf)?.skip_all(),
            ValueKind::U8Map => {
                MapDeserializer::<u8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::I8Map => {
                MapDeserializer::<i8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::U16Map => {
                MapDeserializer::<u16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::I16Map => {
                MapDeserializer::<i16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::U32Map => {
                MapDeserializer::<u32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::I32Map => {
                MapDeserializer::<i32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::U64Map => {
                MapDeserializer::<u64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::I64Map => {
                MapDeserializer::<i64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::StringMap => {
                MapDeserializer::<String>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::UuidMap => {
                MapDeserializer::<Uuid>::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::U8Set => SetDeserializer::<u8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I8Set => SetDeserializer::<i8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U16Set => SetDeserializer::<u16>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I16Set => SetDeserializer::<i16>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U32Set => SetDeserializer::<u32>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I32Set => SetDeserializer::<i32>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U64Set => SetDeserializer::<u64>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I64Set => SetDeserializer::<i64>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::StringSet => {
                SetDeserializer::<String>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::UuidSet => SetDeserializer::<Uuid>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::Struct => {
                StructDeserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }
            ValueKind::Enum => {
                EnumDeserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }
        }
    }

    pub fn deserialize_none(self) -> Result<(), DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::None)
    }

    pub fn deserialize_some<T: Deserialize>(mut self) -> Result<T, DeserializeError> {
        self.increment_depth()?;
        self.buf.ensure_discriminant_u8(ValueKind::Some)?;
        T::deserialize(self)
    }

    pub fn deserialize_option<T: Deserialize>(mut self) -> Result<Option<T>, DeserializeError> {
        self.increment_depth()?;
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::Some => T::deserialize(self).map(Some),
            ValueKind::None => Ok(None),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }

    pub fn deserialize_bool(self) -> Result<bool, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Bool)?;
        self.buf.try_get_u8().map(|v| v != 0)
    }

    pub fn deserialize_u8(self) -> Result<u8, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::U8)?;
        self.buf.try_get_u8()
    }

    pub fn deserialize_i8(self) -> Result<i8, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::I8)?;
        self.buf.try_get_i8()
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
        self.buf.try_get_u32_le().map(f32::from_bits)
    }

    pub fn deserialize_f64(self) -> Result<f64, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::F64)?;
        self.buf.try_get_u64_le().map(f64::from_bits)
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
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn deserialize_object_id(self) -> Result<ObjectId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ObjectId)?;
        let mut bytes = uuid::Bytes::default();

        self.buf.try_copy_to_slice(&mut bytes)?;
        let uuid = ObjectUuid(Uuid::from_bytes(bytes));

        self.buf.try_copy_to_slice(&mut bytes)?;
        let cookie = ObjectCookie(Uuid::from_bytes(bytes));

        Ok(ObjectId::new(uuid, cookie))
    }

    pub fn deserialize_service_id(self) -> Result<ServiceId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ServiceId)?;
        let mut bytes = uuid::Bytes::default();

        self.buf.try_copy_to_slice(&mut bytes)?;
        let object_uuid = ObjectUuid(Uuid::from_bytes(bytes));

        self.buf.try_copy_to_slice(&mut bytes)?;
        let object_cookie = ObjectCookie(Uuid::from_bytes(bytes));

        self.buf.try_copy_to_slice(&mut bytes)?;
        let service_uuid = ServiceUuid(Uuid::from_bytes(bytes));

        self.buf.try_copy_to_slice(&mut bytes)?;
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

    pub fn deserialize_vec_extend<V, T>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        V: Extend<T>,
        T: Deserialize,
    {
        self.deserialize_vec()?.deserialize_extend(vec)
    }

    pub fn deserialize_vec_extend_new<V, T>(self) -> Result<V, DeserializeError>
    where
        V: Extend<T> + Default,
        T: Deserialize,
    {
        let mut vec = V::default();
        self.deserialize_vec()?.deserialize_extend(&mut vec)?;
        Ok(vec)
    }

    pub fn deserialize_bytes(self) -> Result<BytesDeserializer<'a, 'b>, DeserializeError> {
        BytesDeserializer::new(self.buf)
    }

    pub fn deserialize_bytes_to_vec(self) -> Result<Vec<u8>, DeserializeError> {
        BytesDeserializer::new(self.buf)?.deserialize_all_to_vec()
    }

    pub fn deserialize_map<K: DeserializeKey>(
        self,
    ) -> Result<MapDeserializer<'a, 'b, K>, DeserializeError> {
        MapDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_map_extend<T, K, V>(self, map: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<(K, V)>,
        K: DeserializeKey,
        V: Deserialize,
    {
        MapDeserializer::new(self.buf, self.depth)?.deserialize_extend(map)
    }

    pub fn deserialize_map_extend_new<T, K, V>(self) -> Result<T, DeserializeError>
    where
        T: Extend<(K, V)> + Default,
        K: DeserializeKey,
        V: Deserialize,
    {
        let mut map = T::default();
        MapDeserializer::new(self.buf, self.depth)?.deserialize_extend(&mut map)?;
        Ok(map)
    }

    pub fn deserialize_set<T: DeserializeKey>(
        self,
    ) -> Result<SetDeserializer<'a, 'b, T>, DeserializeError> {
        SetDeserializer::new(self.buf)
    }

    pub fn deserialize_set_extend<T, S>(self, set: &mut S) -> Result<(), DeserializeError>
    where
        S: Extend<T>,
        T: DeserializeKey,
    {
        SetDeserializer::new(self.buf)?.deserialize_extend(set)
    }

    pub fn deserialize_set_extend_new<T, S>(self) -> Result<S, DeserializeError>
    where
        S: Extend<T> + Default,
        T: DeserializeKey,
    {
        let mut set = S::default();
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
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }

    pub fn deserialize_receiver(self) -> Result<ChannelCookie, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Receiver)?;
        let mut bytes = uuid::Bytes::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }
}

#[derive(Debug)]
pub struct VecDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
}

impl<'a, 'b> VecDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec)?;
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

    pub fn deserialize_element<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: Deserialize,
    {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            T::deserialize(Deserializer::new(self.buf, self.depth)?)
        }
    }

    pub fn deserialize_extend<V, T>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        V: Extend<T>,
        T: Deserialize,
    {
        while !self.is_empty() {
            let elem = self.deserialize_element()?;
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
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
        if self.is_empty() {
            Ok(t)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
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
        self.skip()?;
        Ok(t)
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
        Ok(Self { buf, len })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize(&mut self, dst: &mut [u8]) -> Result<(), DeserializeError> {
        if dst.len() <= self.len as usize {
            self.buf.try_copy_to_slice(dst)?;
            self.len -= dst.len() as u32;
            Ok(())
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_to_vec(&mut self, len: usize) -> Result<Vec<u8>, DeserializeError> {
        if self.len as usize >= len {
            let bytes = self.buf.try_copy_to_bytes(self.len as usize)?;
            self.len -= len as u32;
            Ok(Vec::from(bytes))
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_all_to_vec(mut self) -> Result<Vec<u8>, DeserializeError> {
        self.deserialize_to_vec(self.len as usize)
    }

    pub fn skip(&mut self, len: usize) -> Result<(), DeserializeError> {
        if self.len as usize >= len {
            self.buf.try_skip(len)?;
            self.len -= len as u32;
            Ok(())
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn skip_all(mut self) -> Result<(), DeserializeError> {
        self.skip(self.len as usize)
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        if self.is_empty() {
            Ok(t)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
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
        self.skip_all()?;
        Ok(t)
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip_all()?;
        f()
    }
}

#[derive(Debug)]
pub struct MapDeserializer<'a, 'b, K: DeserializeKey> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: DeserializeKey> MapDeserializer<'a, 'b, K> {
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

    pub fn deserialize_element(
        &mut self,
    ) -> Result<ElementDeserializer<'_, 'b, K>, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            ElementDeserializer::new(self.buf, self.depth)
        }
    }

    pub fn deserialize_extend<T, V>(mut self, map: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<(K, V)>,
        V: Deserialize,
    {
        while !self.is_empty() {
            let kv = self.deserialize_element()?.deserialize()?;
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
        if self.is_empty() {
            Ok(t)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
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
        self.skip()?;
        Ok(t)
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
pub struct ElementDeserializer<'a, 'b, K: DeserializeKey> {
    buf: &'a mut &'b [u8],
    key: K,
    depth: u8,
}

impl<'a, 'b, K: DeserializeKey> ElementDeserializer<'a, 'b, K> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let key = K::Impl::deserialize_key(buf)?;
        let key = K::try_from_impl(key)?;
        Ok(Self { buf, key, depth })
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn deserialize<T: Deserialize>(self) -> Result<(K, T), DeserializeError> {
        let value = T::deserialize(Deserializer::new(self.buf, self.depth)?)?;
        Ok((self.key, value))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }
}

#[derive(Debug)]
pub struct SetDeserializer<'a, 'b, T: DeserializeKey> {
    buf: &'a mut &'b [u8],
    len: u32,
    _key: PhantomData<T>,
}

impl<'a, 'b, T: DeserializeKey> SetDeserializer<'a, 'b, T> {
    fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        T::Impl::deserialize_set_value_kind(buf)?;
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

    pub fn deserialize_element(&mut self) -> Result<T, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            let key = T::Impl::deserialize_key(self.buf)?;
            T::try_from_impl(key)
        }
    }

    pub fn deserialize_extend<S>(mut self, set: &mut S) -> Result<(), DeserializeError>
    where
        S: Extend<T>,
    {
        while !self.is_empty() {
            let kv = self.deserialize_element()?;
            set.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            T::Impl::skip(self.buf)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.skip_element()?;
        }

        Ok(())
    }

    pub fn finish<T2>(self, t: T2) -> Result<T2, DeserializeError> {
        if self.is_empty() {
            Ok(t)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn finish_with<T2, F>(self, f: F) -> Result<T2, DeserializeError>
    where
        F: FnOnce() -> Result<T2, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T2>(self, t: T2) -> Result<T2, DeserializeError> {
        self.skip()?;
        Ok(t)
    }

    pub fn skip_and_finish_with<T2, F>(self, f: F) -> Result<T2, DeserializeError>
    where
        F: FnOnce() -> Result<T2, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}

#[derive(Debug)]
pub struct StructDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    num_fields: u32,
    depth: u8,
    unknown_fields: UnknownFields,
}

impl<'a, 'b> StructDeserializer<'a, 'b> {
    fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Struct)?;
        Self::new_without_value_kind(buf, depth)
    }

    fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let num_fields = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            num_fields,
            depth,
            unknown_fields: UnknownFields::new(),
        })
    }

    pub fn remaining_fields(&self) -> usize {
        self.num_fields as usize
    }

    pub fn has_more_fields(&self) -> bool {
        self.num_fields > 0
    }

    pub fn deserialize_field(&mut self) -> Result<FieldDeserializer<'_, 'b>, DeserializeError> {
        if self.has_more_fields() {
            self.num_fields -= 1;
            FieldDeserializer::new(self.buf, self.depth, &mut self.unknown_fields)
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_specific_field<T: Deserialize>(
        &mut self,
        id: impl Into<u32>,
    ) -> Result<T, DeserializeError> {
        let field = self.deserialize_field()?;

        if field.id() == id.into() {
            field.deserialize()
        } else {
            Err(DeserializeError::InvalidSerialization)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.has_more_fields() {
            self.deserialize_field()?.skip()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        if self.has_more_fields() {
            Err(DeserializeError::MoreElementsRemain)
        } else {
            Ok(t)
        }
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        if self.has_more_fields() {
            Err(DeserializeError::MoreElementsRemain)
        } else {
            f(self.unknown_fields)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip()?;
        Ok(t)
    }

    pub fn skip_and_finish_with<T, F>(mut self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        while self.has_more_fields() {
            self.deserialize_field()?.skip()?;
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

    pub fn deserialize<T: Deserialize>(self) -> Result<T, DeserializeError> {
        T::deserialize(Deserializer::new(self.buf, self.depth)?)
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }

    pub fn add_to_unknown_fields(self) -> Result<(), DeserializeError> {
        let value = SerializedValue::deserialize(Deserializer::new(self.buf, self.depth)?)?;
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

    pub fn deserialize<T: Deserialize>(self) -> Result<T, DeserializeError> {
        T::deserialize(Deserializer::new(self.buf, self.depth)?)
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
