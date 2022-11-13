use crate::error::SerializeError;
use crate::ids::{ChannelCookie, ObjectId, ServiceId};
use crate::serialize_key::SerializeKey;
use crate::util::BufMutExt;
use crate::value::ValueKind;
use bytes::{BufMut, BytesMut};
use std::fmt;
use std::marker::PhantomData;
use uuid::Uuid;

pub trait Serialize {
    fn serialize<B: BufMut>(&self, serializer: Serializer<B>) -> Result<(), SerializeError>;
}

#[derive(Debug)]
pub struct Serializer<'a, B: BufMut> {
    buf: &'a mut B,
}

impl<'a> Serializer<'a, BytesMut> {
    pub fn with_message_header(buf: &'a mut BytesMut) -> Result<Self, SerializeError> {
        let empty_header = [0, 0, 0, 0, 0];
        buf.try_put_slice(empty_header)?;
        Ok(Self::new(buf))
    }
}

impl<'a, B: BufMut> Serializer<'a, B> {
    pub fn new(buf: &'a mut B) -> Self {
        Self { buf }
    }

    pub fn serialize_none(self) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::None)
    }

    pub fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Some)?;
        value.serialize(self)
    }

    pub fn serialize_bool(self, value: bool) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Bool)?;
        self.buf.try_put_u8(value.into())?;
        Ok(())
    }

    pub fn serialize_u8(self, value: u8) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::U8)?;
        self.buf.try_put_u8(value)?;
        Ok(())
    }

    pub fn serialize_i8(self, value: i8) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::I8)?;
        self.buf.try_put_i8(value)?;
        Ok(())
    }

    pub fn serialize_u16(self, value: u16) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::U16)?;
        self.buf.try_put_varint_u16_le(value)?;
        Ok(())
    }

    pub fn serialize_i16(self, value: i16) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::I16)?;
        self.buf.try_put_varint_i16_le(value)?;
        Ok(())
    }

    pub fn serialize_u32(self, value: u32) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::U32)?;
        self.buf.try_put_varint_u32_le(value)?;
        Ok(())
    }

    pub fn serialize_i32(self, value: i32) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::I32)?;
        self.buf.try_put_varint_i32_le(value)?;
        Ok(())
    }

    pub fn serialize_u64(self, value: u64) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::U64)?;
        self.buf.try_put_varint_u64_le(value)?;
        Ok(())
    }

    pub fn serialize_i64(self, value: i64) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::I64)?;
        self.buf.try_put_varint_i64_le(value)?;
        Ok(())
    }

    pub fn serialize_f32(self, value: f32) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::F32)?;
        self.buf.try_put_u32_le(value.to_bits())?;
        Ok(())
    }

    pub fn serialize_f64(self, value: f64) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::F64)?;
        self.buf.try_put_u64_le(value.to_bits())?;
        Ok(())
    }

    pub fn serialize_string(self, value: impl AsRef<str>) -> Result<(), SerializeError> {
        let value = value.as_ref();

        if value.len() <= u32::MAX as usize {
            self.buf.try_put_discriminant_u8(ValueKind::String)?;
            self.buf.try_put_varint_u32_le(value.len() as u32)?;
            self.buf.try_put_slice(value)?;
            Ok(())
        } else {
            Err(SerializeError)
        }
    }

    pub fn serialize_uuid(self, value: Uuid) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Uuid)?;
        self.buf.try_put_slice(value)?;
        Ok(())
    }

    pub fn serialize_object_id(self, value: ObjectId) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::ObjectId)?;
        self.buf.try_put_slice(value.uuid.0)?;
        self.buf.try_put_slice(value.cookie.0)?;
        Ok(())
    }

    pub fn serialize_service_id(self, value: ServiceId) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::ServiceId)?;
        self.buf.try_put_slice(value.object_id.uuid.0)?;
        self.buf.try_put_slice(value.object_id.cookie.0)?;
        self.buf.try_put_slice(value.uuid.0)?;
        self.buf.try_put_slice(value.cookie.0)?;
        Ok(())
    }

    pub fn serialize_vec(self, num_elems: usize) -> Result<VecSerializer<'a, B>, SerializeError> {
        VecSerializer::new(self.buf, num_elems)
    }

    pub fn serialize_vec_iter<T>(self, vec: T) -> Result<(), SerializeError>
    where
        T: IntoIterator,
        T::Item: Serialize,
        T::IntoIter: ExactSizeIterator,
    {
        let vec = vec.into_iter();
        let mut serializer = self.serialize_vec(vec.len())?;

        for elem in vec {
            serializer.serialize_element(&elem)?;
        }

        serializer.finish()
    }

    pub fn serialize_bytes(self, value: impl AsRef<[u8]>) -> Result<(), SerializeError> {
        let value = value.as_ref();

        if value.len() <= u32::MAX as usize {
            self.buf.try_put_discriminant_u8(ValueKind::Bytes)?;
            self.buf.try_put_varint_u32_le(value.len() as u32)?;
            self.buf.try_put_slice(value)?;
            Ok(())
        } else {
            Err(SerializeError)
        }
    }

    pub fn serialize_map<K: SerializeKey + ?Sized>(
        self,
        num_elems: usize,
    ) -> Result<MapSerializer<'a, B, K>, SerializeError> {
        MapSerializer::new(self.buf, num_elems)
    }

    pub fn serialize_map_iter<T, K, V>(self, map: T) -> Result<(), SerializeError>
    where
        T: IntoIterator<Item = (K, V)>,
        T::IntoIter: ExactSizeIterator,
        K: SerializeKey,
        V: Serialize,
    {
        let map = map.into_iter();
        let mut serializer = self.serialize_map(map.len())?;

        for (key, value) in map {
            serializer.serialize_element(&key, &value)?;
        }

        serializer.finish()
    }

    pub fn serialize_set<T: SerializeKey + ?Sized>(
        self,
        num_elems: usize,
    ) -> Result<SetSerializer<'a, B, T>, SerializeError> {
        SetSerializer::new(self.buf, num_elems)
    }

    pub fn serialize_set_iter<T>(self, set: T) -> Result<(), SerializeError>
    where
        T: IntoIterator,
        T::Item: SerializeKey,
        T::IntoIter: ExactSizeIterator,
    {
        let set = set.into_iter();
        let mut serializer = self.serialize_set(set.len())?;

        for value in set {
            serializer.serialize_element(&value)?;
        }

        serializer.finish()
    }

    pub fn serialize_struct(
        self,
        num_fields: usize,
    ) -> Result<StructSerializer<'a, B>, SerializeError> {
        StructSerializer::new(self.buf, num_fields)
    }

    pub fn serialize_enum<T: Serialize + ?Sized>(
        self,
        variant: u32,
        value: &T,
    ) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Enum)?;
        self.buf.try_put_varint_u32_le(variant)?;
        value.serialize(self)
    }

    pub fn serialize_sender(self, value: ChannelCookie) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Sender)?;
        self.buf.try_put_slice(value.0)?;
        Ok(())
    }

    pub fn serialize_receiver(self, value: ChannelCookie) -> Result<(), SerializeError> {
        self.buf.try_put_discriminant_u8(ValueKind::Receiver)?;
        self.buf.try_put_slice(value.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct VecSerializer<'a, B: BufMut> {
    buf: &'a mut B,
    num_elems: usize,
}

impl<'a, B: BufMut> VecSerializer<'a, B> {
    fn new(buf: &'a mut B, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.try_put_discriminant_u8(ValueKind::Vec)?;
            buf.try_put_varint_u32_le(num_elems as u32)?;
            Ok(Self { buf, num_elems })
        } else {
            Err(SerializeError)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize_element<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            value.serialize(Serializer::new(self.buf))?;
            Ok(self)
        } else {
            Err(SerializeError)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError)
        }
    }
}

pub struct MapSerializer<'a, B: BufMut, K: SerializeKey + ?Sized> {
    buf: &'a mut B,
    num_elems: usize,
    _key: PhantomData<K>,
}

impl<'a, B: BufMut, K: SerializeKey + ?Sized> MapSerializer<'a, B, K> {
    fn new(mut buf: &'a mut B, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            K::serialize_map_value_kind(&mut buf)?;
            buf.try_put_varint_u32_le(num_elems as u32)?;

            Ok(Self {
                buf,
                num_elems,
                _key: PhantomData,
            })
        } else {
            Err(SerializeError)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize_element<T: Serialize + ?Sized>(
        &mut self,
        key: &K,
        value: &T,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            key.serialize_key(self.buf)?;
            value.serialize(Serializer::new(self.buf))?;
            Ok(self)
        } else {
            Err(SerializeError)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError)
        }
    }
}

impl<'a, B: BufMut + fmt::Debug, K: SerializeKey + ?Sized> fmt::Debug for MapSerializer<'a, B, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);

        f.finish()
    }
}

pub struct SetSerializer<'a, B: BufMut, T: SerializeKey + ?Sized> {
    buf: &'a mut B,
    num_elems: usize,
    _key: PhantomData<T>,
}

impl<'a, B: BufMut, T: SerializeKey + ?Sized> SetSerializer<'a, B, T> {
    fn new(mut buf: &'a mut B, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            T::serialize_set_value_kind(&mut buf)?;
            buf.try_put_varint_u32_le(num_elems as u32)?;

            Ok(Self {
                buf,
                num_elems,
                _key: PhantomData,
            })
        } else {
            Err(SerializeError)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize_element(&mut self, value: &T) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            value.serialize_key(self.buf)?;
            Ok(self)
        } else {
            Err(SerializeError)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError)
        }
    }
}

impl<'a, B: BufMut + fmt::Debug, T: SerializeKey + ?Sized> fmt::Debug for SetSerializer<'a, B, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SetSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);

        f.finish()
    }
}

#[derive(Debug)]
pub struct StructSerializer<'a, B: BufMut> {
    buf: &'a mut B,
    num_fields: usize,
}

impl<'a, B: BufMut> StructSerializer<'a, B> {
    fn new(buf: &'a mut B, num_fields: usize) -> Result<Self, SerializeError> {
        if num_fields <= u32::MAX as usize {
            buf.try_put_discriminant_u8(ValueKind::Struct)?;
            buf.try_put_varint_u32_le(num_fields as u32)?;
            Ok(Self { buf, num_fields })
        } else {
            Err(SerializeError)
        }
    }

    pub fn remaining_fields(&self) -> usize {
        self.num_fields
    }

    pub fn requires_additional_fields(&self) -> bool {
        self.num_fields > 0
    }

    pub fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        id: u32,
        value: &T,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_fields > 0 {
            self.num_fields -= 1;
            self.buf.try_put_varint_u32_le(id)?;
            value.serialize(Serializer::new(self.buf))?;
            Ok(self)
        } else {
            Err(SerializeError)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_fields == 0 {
            Ok(())
        } else {
            Err(SerializeError)
        }
    }
}
