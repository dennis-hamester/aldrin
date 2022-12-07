use crate::buf_ext::BufExt;
use crate::error::DeserializeError;
use crate::value::ValueKind;
use bytes::Buf;
use std::mem;
use uuid::Uuid;

pub trait Sealed: Sized {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError>;
    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;
    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;
    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;
}

pub trait DeserializeKey: Sealed + Sized {}

impl Sealed for u8 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_u8()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U8Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U8Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(1)
    }
}

impl DeserializeKey for u8 {}

impl Sealed for i8 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_i8()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I8Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I8Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(1)
    }
}

impl DeserializeKey for i8 {}

impl Sealed for u16 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_u16_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U16Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U16Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<u16>() }>()
    }
}

impl DeserializeKey for u16 {}

impl Sealed for i16 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_i16_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I16Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I16Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<i16>() }>()
    }
}

impl DeserializeKey for i16 {}

impl Sealed for u32 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_u32_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U32Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U32Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<u32>() }>()
    }
}

impl DeserializeKey for u32 {}

impl Sealed for i32 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_i32_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I32Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I32Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<i32>() }>()
    }
}

impl DeserializeKey for i32 {}

impl Sealed for u64 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_u64_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U64Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U64Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<u64>() }>()
    }
}

impl DeserializeKey for u64 {}

impl Sealed for i64 {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        buf.try_get_varint_i64_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I64Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I64Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<i64>() }>()
    }
}

impl DeserializeKey for i64 {}

impl Sealed for String {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        let bytes = buf.try_copy_to_bytes(len)?.into();
        String::from_utf8(bytes).map_err(|_| DeserializeError)
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::StringMap)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::StringSet)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        buf.try_skip(len)
    }
}

impl DeserializeKey for String {}

impl Sealed for Uuid {
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self, DeserializeError> {
        let mut bytes = Default::default();
        buf.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::UuidMap)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::UuidSet)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(16)
    }
}

impl DeserializeKey for Uuid {}
