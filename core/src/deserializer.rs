mod bytes;
mod enum_;
mod map;
mod set;
mod struct_;
mod vec;

use crate::buf_ext::ValueBufExt;
use crate::tags::{self, KeyTag, Tag};
use crate::{
    ChannelCookie, Deserialize, DeserializeError, DeserializeKey, ObjectCookie, ObjectId,
    ObjectUuid, SerializedValueSlice, ServiceCookie, ServiceId, ServiceUuid, ValueKind,
    MAX_VALUE_DEPTH,
};
use ::bytes::Buf;
use uuid::Uuid;

pub use self::bytes::{Bytes1Deserializer, Bytes2Deserializer, BytesDeserializer};
pub use enum_::EnumDeserializer;
pub use map::{Map1Deserializer, Map2Deserializer, MapDeserializer, MapElementDeserializer};
pub use set::{Set1Deserializer, Set2Deserializer, SetDeserializer};
pub use struct_::{
    FieldDeserializer, Struct1Deserializer, Struct2Deserializer, StructDeserializer,
};
pub use vec::{Vec1Deserializer, Vec2Deserializer, VecDeserializer};

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

            ValueKind::Bytes1 => Bytes1Deserializer::new_without_value_kind(self.buf)?.skip(),

            ValueKind::U8Map1 => {
                Map1Deserializer::<tags::U8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I8Map1 => {
                Map1Deserializer::<tags::I8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U16Map1 => {
                Map1Deserializer::<tags::U16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I16Map1 => {
                Map1Deserializer::<tags::I16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U32Map1 => {
                Map1Deserializer::<tags::U32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I32Map1 => {
                Map1Deserializer::<tags::I32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U64Map1 => {
                Map1Deserializer::<tags::U64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I64Map1 => {
                Map1Deserializer::<tags::I64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::StringMap1 => {
                Map1Deserializer::<tags::String>::new_without_value_kind(self.buf, self.depth)?
                    .skip()
            }

            ValueKind::UuidMap1 => {
                Map1Deserializer::<tags::Uuid>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U8Set1 => {
                Set1Deserializer::<tags::U8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I8Set1 => {
                Set1Deserializer::<tags::I8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U16Set1 => {
                Set1Deserializer::<tags::U16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I16Set1 => {
                Set1Deserializer::<tags::I16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U32Set1 => {
                Set1Deserializer::<tags::U32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I32Set1 => {
                Set1Deserializer::<tags::I32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U64Set1 => {
                Set1Deserializer::<tags::U64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I64Set1 => {
                Set1Deserializer::<tags::I64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::StringSet1 => {
                Set1Deserializer::<tags::String>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::UuidSet1 => {
                Set1Deserializer::<tags::Uuid>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::Struct1 => {
                Struct1Deserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Enum => {
                EnumDeserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Vec2 => {
                Vec2Deserializer::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::Bytes2 => Bytes2Deserializer::new_without_value_kind(self.buf)?.skip(),

            ValueKind::U8Map2 => {
                Map2Deserializer::<tags::U8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I8Map2 => {
                Map2Deserializer::<tags::I8>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U16Map2 => {
                Map2Deserializer::<tags::U16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I16Map2 => {
                Map2Deserializer::<tags::I16>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U32Map2 => {
                Map2Deserializer::<tags::U32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I32Map2 => {
                Map2Deserializer::<tags::I32>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U64Map2 => {
                Map2Deserializer::<tags::U64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::I64Map2 => {
                Map2Deserializer::<tags::I64>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::StringMap2 => {
                Map2Deserializer::<tags::String>::new_without_value_kind(self.buf, self.depth)?
                    .skip()
            }

            ValueKind::UuidMap2 => {
                Map2Deserializer::<tags::Uuid>::new_without_value_kind(self.buf, self.depth)?.skip()
            }

            ValueKind::U8Set2 => {
                Set2Deserializer::<tags::U8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I8Set2 => {
                Set2Deserializer::<tags::I8>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U16Set2 => {
                Set2Deserializer::<tags::U16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I16Set2 => {
                Set2Deserializer::<tags::I16>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U32Set2 => {
                Set2Deserializer::<tags::U32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I32Set2 => {
                Set2Deserializer::<tags::I32>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::U64Set2 => {
                Set2Deserializer::<tags::U64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::I64Set2 => {
                Set2Deserializer::<tags::I64>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::StringSet2 => {
                Set2Deserializer::<tags::String>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::UuidSet2 => {
                Set2Deserializer::<tags::Uuid>::new_without_value_kind(self.buf)?.skip()
            }

            ValueKind::Struct2 => {
                Struct2Deserializer::new_without_value_kind(self.buf, self.depth)?.skip()
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

    pub fn deserialize_bytes1(self) -> Result<Bytes1Deserializer<'a, 'b>, DeserializeError> {
        Bytes1Deserializer::new(self.buf)
    }

    pub fn deserialize_bytes1_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        self.deserialize_bytes1()?.deserialize_extend(bytes)
    }

    pub fn deserialize_bytes1_extend_new<T>(self) -> Result<T, DeserializeError>
    where
        T: Default + Extend<u8>,
    {
        let mut bytes = T::default();
        self.deserialize_bytes1()?.deserialize_extend(&mut bytes)?;
        Ok(bytes)
    }

    pub fn deserialize_bytes2(self) -> Result<Bytes2Deserializer<'a, 'b>, DeserializeError> {
        Bytes2Deserializer::new(self.buf)
    }

    pub fn deserialize_bytes2_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        self.deserialize_bytes2()?.deserialize_extend(bytes)
    }

    pub fn deserialize_bytes2_extend_new<T>(self) -> Result<T, DeserializeError>
    where
        T: Default + Extend<u8>,
    {
        let mut bytes = T::default();
        self.deserialize_bytes2()?.deserialize_extend(&mut bytes)?;
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

    pub fn deserialize_map1<K: KeyTag>(
        self,
    ) -> Result<Map1Deserializer<'a, 'b, K>, DeserializeError> {
        Map1Deserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_map1_extend<K, L, T, U, V>(self, map: &mut V) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        Map1Deserializer::new(self.buf, self.depth)?.deserialize_extend(map)
    }

    pub fn deserialize_map1_extend_new<K, L, T, U, V>(self) -> Result<V, DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<(L, U)>,
    {
        let mut map = V::default();
        self.deserialize_map1_extend(&mut map)?;
        Ok(map)
    }

    pub fn deserialize_map2<K: KeyTag>(
        self,
    ) -> Result<Map2Deserializer<'a, 'b, K>, DeserializeError> {
        Map2Deserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_map2_extend<K, L, T, U, V>(self, map: &mut V) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        Map2Deserializer::new(self.buf, self.depth)?.deserialize_extend(map)
    }

    pub fn deserialize_map2_extend_new<K, L, T, U, V>(self) -> Result<V, DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Default + Extend<(L, U)>,
    {
        let mut map = V::default();
        self.deserialize_map2_extend(&mut map)?;
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
        self.deserialize_set()?.deserialize_extend(set)
    }

    pub fn deserialize_set_extend_new<K, T, U>(self) -> Result<U, DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Default + Extend<T>,
    {
        let mut set = U::default();
        self.deserialize_set_extend(&mut set)?;
        Ok(set)
    }

    pub fn deserialize_set1<K: KeyTag>(
        self,
    ) -> Result<Set1Deserializer<'a, 'b, K>, DeserializeError> {
        Set1Deserializer::new(self.buf)
    }

    pub fn deserialize_set1_extend<K, T, U>(self, set: &mut U) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        self.deserialize_set1()?.deserialize_extend(set)
    }

    pub fn deserialize_set1_extend_new<K, T, U>(self) -> Result<U, DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Default + Extend<T>,
    {
        let mut set = U::default();
        self.deserialize_set1_extend(&mut set)?;
        Ok(set)
    }

    pub fn deserialize_set2<K: KeyTag>(
        self,
    ) -> Result<Set2Deserializer<'a, 'b, K>, DeserializeError> {
        Set2Deserializer::new(self.buf)
    }

    pub fn deserialize_set2_extend<K, T, U>(self, set: &mut U) -> Result<(), DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        self.deserialize_set2()?.deserialize_extend(set)
    }

    pub fn deserialize_set2_extend_new<K, T, U>(self) -> Result<U, DeserializeError>
    where
        K: KeyTag,
        T: DeserializeKey<K>,
        U: Default + Extend<T>,
    {
        let mut set = U::default();
        self.deserialize_set2_extend(&mut set)?;
        Ok(set)
    }

    pub fn deserialize_struct(self) -> Result<StructDeserializer<'a, 'b>, DeserializeError> {
        StructDeserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_struct1(self) -> Result<Struct1Deserializer<'a, 'b>, DeserializeError> {
        Struct1Deserializer::new(self.buf, self.depth)
    }

    pub fn deserialize_struct2(self) -> Result<Struct2Deserializer<'a, 'b>, DeserializeError> {
        Struct2Deserializer::new(self.buf, self.depth)
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
