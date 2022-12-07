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

impl<'a, T: Sealed + ?Sized> Sealed for &'a T {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        (*self).serialize_key(buf)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_map_value_kind(buf)
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        T::serialize_set_value_kind(buf)
    }
}

impl<'a, T: SerializeKey + ?Sized> SerializeKey for &'a T {}

impl Sealed for u8 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_u8(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U8Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U8Set).unwrap()
    }
}

impl SerializeKey for u8 {}

impl Sealed for i8 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_i8(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I8Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I8Set).unwrap()
    }
}

impl SerializeKey for i8 {}

impl Sealed for u16 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_u16_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U16Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U16Set).unwrap()
    }
}

impl SerializeKey for u16 {}

impl Sealed for i16 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_i16_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I16Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I16Set).unwrap()
    }
}

impl SerializeKey for i16 {}

impl Sealed for u32 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_u32_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U32Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U32Set).unwrap()
    }
}

impl SerializeKey for u32 {}

impl Sealed for i32 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_i32_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I32Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I32Set).unwrap()
    }
}

impl SerializeKey for i32 {}

impl Sealed for u64 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_u64_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U64Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::U64Set).unwrap()
    }
}

impl SerializeKey for u64 {}

impl Sealed for i64 {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        buf.try_put_varint_i64_le(*self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I64Map).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::I64Set).unwrap()
    }
}

impl SerializeKey for i64 {}

impl Sealed for str {
    fn serialize_key<B: BufMut>(&self, buf: &mut B) -> Result<(), SerializeError> {
        if self.len() <= u32::MAX as usize {
            buf.try_put_varint_u32_le(self.len() as u32)?;
            buf.try_put_slice(self)?;
            Ok(())
        } else {
            Err(SerializeError)
        }
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::StringMap).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::StringSet).unwrap()
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
        buf.try_put_slice(self)
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::UuidMap).unwrap()
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.try_put_discriminant_u8(ValueKind::UuidSet).unwrap()
    }
}

impl SerializeKey for Uuid {}
