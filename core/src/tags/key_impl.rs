use super::{String, Uuid, I16, I32, I64, I8, U16, U32, U64, U8};
use crate::buf_ext::{BufMutExt, ValueBufExt};
use crate::{DeserializeError, SerializeError, ValueKind};
use bytes::{Buf, BufMut};
use std::borrow::Cow;
use std::mem;

pub trait Sealed {}

pub trait KeyTagImpl: Sized + Sealed {
    type Key<'a>;

    // Not part of the public API.
    #[doc(hidden)]
    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn serialize_map_value_kind<B: BufMut>(buf: &mut B);

    // Not part of the public API.
    #[doc(hidden)]
    fn serialize_set_value_kind<B: BufMut>(buf: &mut B);

    // Not part of the public API.
    #[doc(hidden)]
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;
}

impl Sealed for U8 {}

impl KeyTagImpl for U8 {
    type Key<'a> = u8;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_u8(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U8Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_u8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
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

impl Sealed for I8 {}

impl KeyTagImpl for I8 {
    type Key<'a> = i8;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_i8(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I8Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_i8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
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

impl Sealed for U16 {}

impl KeyTagImpl for U16 {
    type Key<'a> = u16;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u16_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U16Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u16_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U16Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U16Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for I16 {}

impl KeyTagImpl for I16 {
    type Key<'a> = i16;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i16_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I16Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i16_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I16Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I16Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for U32 {}

impl KeyTagImpl for U32 {
    type Key<'a> = u32;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u32_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U32Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u32_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U32Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U32Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for I32 {}

impl KeyTagImpl for I32 {
    type Key<'a> = i32;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i32_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I32Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i32_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I32Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I32Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for U64 {}

impl KeyTagImpl for U64 {
    type Key<'a> = u64;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u64_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::U64Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u64_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U64Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::U64Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for I64 {}

impl KeyTagImpl for I64 {
    type Key<'a> = i64;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i64_le(key);
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Map);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::I64Set);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i64_le()
    }

    fn deserialize_map_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I64Map)
    }

    fn deserialize_set_value_kind<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::I64Set)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }
}

impl Sealed for String {}

impl KeyTagImpl for String {
    type Key<'a> = Cow<'a, str>;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        if key.len() <= u32::MAX as usize {
            buf.put_varint_u32_le(key.len() as u32);
            buf.put_slice(key.as_bytes());
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

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        let bytes = buf.try_copy_to_bytes(len)?.into();

        std::string::String::from_utf8(bytes)
            .map(Cow::Owned)
            .map_err(|_| DeserializeError::InvalidSerialization)
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

impl Sealed for Uuid {}

impl KeyTagImpl for Uuid {
    type Key<'a> = uuid::Uuid;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_slice(key.as_bytes());
        Ok(())
    }

    fn serialize_map_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::UuidMap);
    }

    fn serialize_set_value_kind<B: BufMut>(buf: &mut B) {
        buf.put_discriminant_u8(ValueKind::UuidSet);
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        let mut bytes = uuid::Bytes::default();

        buf.try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        Ok(uuid::Uuid::from_bytes(bytes))
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
