use crate::deserialize_key::DeserializeKey;
use crate::error::DeserializeError;
use crate::ids::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use crate::util::BufExt;
use crate::value::ValueKind;
use bytes::Buf;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use uuid::Uuid;

pub trait Deserialize: Sized {
    fn deserialize<B: Buf>(deserializer: Deserializer<B>) -> Result<Self, DeserializeError>;
}

#[derive(Debug)]
pub struct Deserializer<'a, B: Buf> {
    buf: &'a mut B,
}

impl<'a, B: Buf> Deserializer<'a, B> {
    pub fn new(buf: &'a mut B) -> Self {
        Self { buf }
    }

    pub fn with_message_header(buf: &'a mut B) -> Result<Self, DeserializeError> {
        buf.try_skip(9)?;
        Ok(Self { buf })
    }

    pub fn peek_value_kind(&self) -> Result<ValueKind, DeserializeError> {
        self.buf.try_peek_discriminant_u8()
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::None => Ok(()),
            ValueKind::Some => self.skip(),
            ValueKind::Bool | ValueKind::U8 | ValueKind::I8 => self.buf.try_skip(1),
            ValueKind::U16 => self.buf.try_skip_varint_le::<{ mem::size_of::<u16>() }>(),
            ValueKind::I16 => self.buf.try_skip_varint_le::<{ mem::size_of::<i16>() }>(),
            ValueKind::U32 => self.buf.try_skip_varint_le::<{ mem::size_of::<u32>() }>(),
            ValueKind::I32 => self.buf.try_skip_varint_le::<{ mem::size_of::<i32>() }>(),
            ValueKind::U64 => self.buf.try_skip_varint_le::<{ mem::size_of::<u64>() }>(),
            ValueKind::I64 => self.buf.try_skip_varint_le::<{ mem::size_of::<i64>() }>(),
            ValueKind::F32 => self.buf.try_skip(mem::size_of::<f32>()),
            ValueKind::F64 => self.buf.try_skip(mem::size_of::<f64>()),
            ValueKind::String => {
                let len = self.buf.try_get_varint_u32_le()? as usize;
                self.buf.try_skip(len)
            }
            ValueKind::Uuid | ValueKind::Sender | ValueKind::Receiver => {
                self.buf.try_skip(mem::size_of::<Uuid>())
            }
            ValueKind::ObjectId => self.buf.try_skip(2 * mem::size_of::<Uuid>()),
            ValueKind::ServiceId => self.buf.try_skip(4 * mem::size_of::<Uuid>()),
            ValueKind::Vec => VecDeserializer::new_without_value_kind(self.buf)?.skip(),
            ValueKind::Bytes => BytesDeserializer::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U8Map => MapDeserializer::<_, u8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I8Map => MapDeserializer::<_, i8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U16Map => {
                MapDeserializer::<_, u16>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I16Map => {
                MapDeserializer::<_, i16>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::U32Map => {
                MapDeserializer::<_, u32>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I32Map => {
                MapDeserializer::<_, i32>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::U64Map => {
                MapDeserializer::<_, u64>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I64Map => {
                MapDeserializer::<_, i64>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::StringMap => {
                MapDeserializer::<_, String>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::UuidMap => {
                MapDeserializer::<_, Uuid>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::U8Set => SetDeserializer::<_, u8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::I8Set => SetDeserializer::<_, i8>::new_without_value_kind(self.buf)?.skip(),
            ValueKind::U16Set => {
                SetDeserializer::<_, u16>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I16Set => {
                SetDeserializer::<_, i16>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::U32Set => {
                SetDeserializer::<_, u32>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I32Set => {
                SetDeserializer::<_, i32>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::U64Set => {
                SetDeserializer::<_, u64>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::I64Set => {
                SetDeserializer::<_, i64>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::StringSet => {
                SetDeserializer::<_, String>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::UuidSet => {
                SetDeserializer::<_, Uuid>::new_without_value_kind(self.buf)?.skip()
            }
            ValueKind::Struct => StructDeserializer::new_without_value_kind(self.buf)?.skip(),
            ValueKind::Enum => EnumDeserializer::new_without_value_kind(self.buf)?.skip(),
        }
    }

    pub fn deserialize_none(self) -> Result<(), DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::None)
    }

    pub fn deserialize_some<T: Deserialize>(self) -> Result<T, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Some)?;
        T::deserialize(self)
    }

    pub fn deserialize_option<T: Deserialize>(self) -> Result<Option<T>, DeserializeError> {
        match self.buf.try_get_discriminant_u8()? {
            ValueKind::Some => T::deserialize(self).map(Some),
            ValueKind::None => Ok(None),
            _ => Err(DeserializeError),
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
        String::from_utf8(bytes).map_err(|_| DeserializeError)
    }

    pub fn deserialize_uuid(self) -> Result<Uuid, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Uuid)?;
        let mut bytes = Default::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn deserialize_object_id(self) -> Result<ObjectId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ObjectId)?;
        let mut bytes = Default::default();

        self.buf.try_copy_to_slice(&mut bytes)?;
        let uuid = ObjectUuid(Uuid::from_bytes(bytes));

        self.buf.try_copy_to_slice(&mut bytes)?;
        let cookie = ObjectCookie(Uuid::from_bytes(bytes));

        Ok(ObjectId::new(uuid, cookie))
    }

    pub fn deserialize_service_id(self) -> Result<ServiceId, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::ServiceId)?;
        let mut bytes = Default::default();

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

    pub fn deserialize_vec(self) -> Result<VecDeserializer<'a, B>, DeserializeError> {
        VecDeserializer::new(self.buf)
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

    pub fn deserialize_bytes(self) -> Result<BytesDeserializer<'a, B>, DeserializeError> {
        BytesDeserializer::new(self.buf)
    }

    pub fn deserialize_bytes_to_vec(self) -> Result<Vec<u8>, DeserializeError> {
        BytesDeserializer::new(self.buf)?.deserialize_to_vec()
    }

    pub fn deserialize_map<K: DeserializeKey>(
        self,
    ) -> Result<MapDeserializer<'a, B, K>, DeserializeError> {
        MapDeserializer::new(self.buf)
    }

    pub fn deserialize_map_extend<T, K, V>(self, map: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<(K, V)>,
        K: DeserializeKey,
        V: Deserialize,
    {
        MapDeserializer::new(self.buf)?.deserialize_extend(map)
    }

    pub fn deserialize_map_extend_new<T, K, V>(self) -> Result<T, DeserializeError>
    where
        T: Extend<(K, V)> + Default,
        K: DeserializeKey,
        V: Deserialize,
    {
        let mut map = T::default();
        MapDeserializer::new(self.buf)?.deserialize_extend(&mut map)?;
        Ok(map)
    }

    pub fn deserialize_set<T: DeserializeKey>(
        self,
    ) -> Result<SetDeserializer<'a, B, T>, DeserializeError> {
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

    pub fn deserialize_struct(self) -> Result<StructDeserializer<'a, B>, DeserializeError> {
        StructDeserializer::new(self.buf)
    }

    pub fn deserialize_enum(self) -> Result<EnumDeserializer<'a, B>, DeserializeError> {
        EnumDeserializer::new(self.buf)
    }

    pub fn deserialize_sender(self) -> Result<ChannelCookie, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Sender)?;
        let mut bytes = Default::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }

    pub fn deserialize_receiver(self) -> Result<ChannelCookie, DeserializeError> {
        self.buf.ensure_discriminant_u8(ValueKind::Receiver)?;
        let mut bytes = Default::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(ChannelCookie(Uuid::from_bytes(bytes)))
    }
}

#[derive(Debug)]
pub struct VecDeserializer<'a, B: Buf> {
    buf: &'a mut B,
    num_elems: u32,
}

impl<'a, B: Buf> VecDeserializer<'a, B> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let num_elems = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, num_elems })
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn has_more_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn deserialize_element<T: Deserialize>(&mut self) -> Result<T, DeserializeError> {
        if self.has_more_elements() {
            self.num_elems -= 1;
            T::deserialize(Deserializer::new(self.buf))
        } else {
            Err(DeserializeError)
        }
    }

    pub fn deserialize_extend<V, T>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        V: Extend<T>,
        T: Deserialize,
    {
        while self.has_more_elements() {
            let elem = self.deserialize_element()?;
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            Deserializer::new(self.buf).skip()
        } else {
            Err(DeserializeError)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.has_more_elements() {
            self.skip_element()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BytesDeserializer<'a, B: Buf> {
    buf: &'a mut B,
    len: u32,
}

impl<'a, B: Buf> BytesDeserializer<'a, B> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Bytes)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, len })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize(&mut self, mut dst: impl AsMut<[u8]>) -> Result<(), DeserializeError> {
        let dst = dst.as_mut();

        if dst.len() == self.len as usize {
            self.buf.try_copy_to_slice(dst)
        } else {
            Err(DeserializeError)
        }
    }

    pub fn deserialize_to_vec(self) -> Result<Vec<u8>, DeserializeError> {
        self.buf.try_copy_to_bytes(self.len as usize).map(Vec::from)
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        self.buf.try_skip(self.len as usize)
    }
}

#[derive(Debug)]
pub struct MapDeserializer<'a, B: Buf, K: DeserializeKey> {
    buf: &'a mut B,
    num_elems: u32,
    _key: PhantomData<K>,
}

impl<'a, B: Buf, K: DeserializeKey> MapDeserializer<'a, B, K> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        K::deserialize_map_value_kind(buf)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let num_elems = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            num_elems,
            _key: PhantomData,
        })
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn has_more_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn deserialize_element<V: Deserialize>(&mut self) -> Result<(K, V), DeserializeError> {
        if self.has_more_elements() {
            self.num_elems -= 1;
            let key = K::deserialize_key(self.buf)?;
            let value = V::deserialize(Deserializer::new(self.buf))?;
            Ok((key, value))
        } else {
            Err(DeserializeError)
        }
    }

    pub fn deserialize_extend<T, V>(mut self, map: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<(K, V)>,
        V: Deserialize,
    {
        while self.has_more_elements() {
            let kv = self.deserialize_element()?;
            map.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            K::skip(self.buf)?;
            Deserializer::new(self.buf).skip()
        } else {
            Err(DeserializeError)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.has_more_elements() {
            self.skip_element()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SetDeserializer<'a, B: Buf, T: DeserializeKey> {
    buf: &'a mut B,
    num_elems: u32,
    _key: PhantomData<T>,
}

impl<'a, B: Buf, T: DeserializeKey> SetDeserializer<'a, B, T> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        T::deserialize_set_value_kind(buf)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let num_elems = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            num_elems,
            _key: PhantomData,
        })
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn has_more_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn deserialize_element(&mut self) -> Result<T, DeserializeError> {
        if self.has_more_elements() {
            self.num_elems -= 1;
            T::deserialize_key(self.buf)
        } else {
            Err(DeserializeError)
        }
    }

    pub fn deserialize_extend<S>(mut self, set: &mut S) -> Result<(), DeserializeError>
    where
        S: Extend<T>,
    {
        while self.has_more_elements() {
            let kv = self.deserialize_element()?;
            set.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            T::skip(self.buf)
        } else {
            Err(DeserializeError)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.has_more_elements() {
            self.skip_element()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct StructDeserializer<'a, B: Buf> {
    buf: &'a mut B,
    num_fields: u32,
}

impl<'a, B: Buf> StructDeserializer<'a, B> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Struct)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let num_fields = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, num_fields })
    }

    pub fn remaining_fields(&self) -> usize {
        self.num_fields as usize
    }

    pub fn has_more_fields(&self) -> bool {
        self.num_fields > 0
    }

    pub fn deserialize_field(&mut self) -> Result<FieldDeserializer<B>, DeserializeError> {
        if self.has_more_fields() {
            self.num_fields -= 1;
            FieldDeserializer::new(self.buf)
        } else {
            Err(DeserializeError)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.has_more_fields() {
            self.deserialize_field()?.skip()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FieldDeserializer<'a, B: Buf> {
    buf: &'a mut B,
    id: u32,
}

impl<'a, B: Buf> FieldDeserializer<'a, B> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let id = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn deserialize<T: Deserialize>(self) -> Result<T, DeserializeError> {
        T::deserialize(Deserializer::new(self.buf))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf).skip()
    }
}

#[derive(Debug)]
pub struct EnumDeserializer<'a, B: Buf> {
    buf: &'a mut B,
    variant: u32,
}

impl<'a, B: Buf> EnumDeserializer<'a, B> {
    fn new(buf: &'a mut B) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Enum)?;
        Self::new_without_value_kind(buf)
    }

    fn new_without_value_kind(buf: &'a mut B) -> Result<Self, DeserializeError> {
        let variant = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, variant })
    }

    pub fn variant(&self) -> u32 {
        self.variant
    }

    pub fn deserialize<T: Deserialize>(self) -> Result<T, DeserializeError> {
        T::deserialize(Deserializer::new(self.buf))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf).skip()
    }
}
