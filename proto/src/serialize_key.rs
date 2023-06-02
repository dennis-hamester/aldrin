use crate::buf_ext::BufMutExt;
use crate::error::SerializeError;
use crate::value::ValueKind;
use bytes::BufMut;
use uuid::Uuid;

pub trait Sealed {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError>;
    fn serialize_map_value_kind<B: BufMut>(buf: &mut B);
    fn serialize_set_value_kind<B: BufMut>(buf: &mut B);
}

pub trait SerializeKey: Sealed {}

impl<T: Sealed + ?Sized> Sealed for &T {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        (**self).serialize_key(buf)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_map_value_kind(buf)
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_set_value_kind(buf)
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for &T {}

impl<T: Sealed + ?Sized> Sealed for &mut T {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        (**self).serialize_key(buf)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_map_value_kind(buf)
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_set_value_kind(buf)
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for &mut T {}

impl<T: Sealed + ?Sized> Sealed for Box<T> {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        (**self).serialize_key(buf)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_map_value_kind(buf)
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_set_value_kind(buf)
    }
}

impl<T: SerializeKey + ?Sized> SerializeKey for Box<T> {}

impl Sealed for u8 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_u8(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Set);
    }
}

impl SerializeKey for u8 {}

impl Sealed for i8 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_i8(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Set);
    }
}

impl SerializeKey for i8 {}

impl Sealed for u16 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_u16_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Set);
    }
}

impl SerializeKey for u16 {}

impl Sealed for i16 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_i16_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Set);
    }
}

impl SerializeKey for i16 {}

impl Sealed for u32 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u32_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Set);
    }
}

impl SerializeKey for u32 {}

impl Sealed for i32 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i32_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Set);
    }
}

impl SerializeKey for i32 {}

impl Sealed for u64 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u64_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Set);
    }
}

impl SerializeKey for u64 {}

impl Sealed for i64 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i64_le(*self);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Set);
    }
}

impl SerializeKey for i64 {}

impl Sealed for str {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
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

impl SerializeKey for str {}

impl Sealed for String {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        self.as_str().serialize_key(buf)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        str::serialize_map_value_kind(buf);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        str::serialize_set_value_kind(buf);
    }
}

impl SerializeKey for String {}

impl Sealed for Uuid {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
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

impl SerializeKey for Uuid {}
