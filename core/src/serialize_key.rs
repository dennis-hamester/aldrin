use crate::buf_ext::BufMutExt;
use crate::error::SerializeError;
use crate::value::ValueKind;
use bytes::BufMut;
use uuid::Uuid;

pub trait Sealed: Sized {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError>;
    fn serialize_map_value_kind<B: BufMut>(buf: &mut B);
    fn serialize_set_value_kind<B: BufMut>(buf: &mut B);
}

pub trait SerializeKeyImpl: Sealed {}

pub trait SerializeKey {
    type Impl<'a>: SerializeKeyImpl
    where
        Self: 'a;

    fn as_impl(&self) -> Self::Impl<'_>;
}

impl Sealed for u8 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_u8(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Set);
    }
}

impl SerializeKeyImpl for u8 {}

impl SerializeKey for u8 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for i8 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_i8(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Set);
    }
}

impl SerializeKeyImpl for i8 {}

impl SerializeKey for i8 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for u16 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u16_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Set);
    }
}

impl SerializeKeyImpl for u16 {}

impl SerializeKey for u16 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for i16 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i16_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Set);
    }
}

impl SerializeKeyImpl for i16 {}

impl SerializeKey for i16 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for u32 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u32_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Set);
    }
}

impl SerializeKeyImpl for u32 {}

impl SerializeKey for u32 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for i32 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i32_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Set);
    }
}

impl SerializeKeyImpl for i32 {}

impl SerializeKey for i32 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for u64 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u64_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Set);
    }
}

impl SerializeKeyImpl for u64 {}

impl SerializeKey for u64 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for i64 {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i64_le(self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Set);
    }
}

impl SerializeKeyImpl for i64 {}

impl SerializeKey for i64 {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl Sealed for &str {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        if self.len() <= u32::MAX as usize {
            buf.put_varint_u32_le(self.len() as u32);
            buf.put_slice(self.as_bytes());
            Ok(())
        } else {
            Err(SerializeError::Overflow)
        }
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::StringMap);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::StringSet);
    }
}

impl SerializeKeyImpl for &str {}

impl SerializeKey for &str {
    type Impl<'a>
        = Self
    where
        Self: 'a;

    fn as_impl(&self) -> Self::Impl<'_> {
        self
    }
}

impl Sealed for Uuid {
    fn serialize_key<B: BufMut>(self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_slice(self.as_bytes());
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::UuidMap);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::UuidSet);
    }
}

impl SerializeKeyImpl for Uuid {}

impl SerializeKey for Uuid {
    type Impl<'a> = Self;

    fn as_impl(&self) -> Self::Impl<'_> {
        *self
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for &T {
    type Impl<'a>
        = T::Impl<'a>
    where
        Self: 'a;

    fn as_impl(&self) -> Self::Impl<'_> {
        (**self).as_impl()
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for &mut T {
    type Impl<'a>
        = T::Impl<'a>
    where
        Self: 'a;

    fn as_impl(&self) -> Self::Impl<'_> {
        (**self).as_impl()
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for Box<T> {
    type Impl<'a>
        = T::Impl<'a>
    where
        Self: 'a;

    fn as_impl(&self) -> Self::Impl<'_> {
        (**self).as_impl()
    }
}

impl SerializeKey for String {
    type Impl<'a> = &'a str;

    fn as_impl(&self) -> Self::Impl<'_> {
        self
    }
}
