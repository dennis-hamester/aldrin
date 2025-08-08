use super::{String, Uuid, I16, I32, I64, I8, U16, U32, U64, U8};
use crate::buf_ext::{BufMutExt, ValueBufExt};
use crate::{DeserializeError, SerializeError, ValueConversionError, ValueKind};
use bytes::{Buf, BufMut, BytesMut};
use std::borrow::Cow;
use std::mem;

#[allow(unnameable_types)]
pub trait Sealed {}

pub trait KeyTagImpl: Sized + Sealed {
    type Key<'a>;

    // Not part of the public API.
    #[doc(hidden)]
    const VALUE_KIND_MAP1: ValueKind;

    // Not part of the public API.
    #[doc(hidden)]
    const VALUE_KIND_MAP2: ValueKind;

    // Not part of the public API.
    #[doc(hidden)]
    const VALUE_KIND_SET1: ValueKind;

    // Not part of the public API.
    #[doc(hidden)]
    const VALUE_KIND_SET2: ValueKind;

    // Not part of the public API.
    #[doc(hidden)]
    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError>;

    // Not part of the public API.
    #[doc(hidden)]
    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError>;
}

impl Sealed for U8 {}

impl KeyTagImpl for U8 {
    type Key<'a> = u8;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::U8Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::U8Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::U8Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::U8Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_u8(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_u8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(1)
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src
            .try_get_u8()
            .map_err(|_| ValueConversionError::Deserialize(DeserializeError::UnexpectedEoi))?;

        dst.put_u8(key);
        Ok(())
    }
}

impl Sealed for I8 {}

impl KeyTagImpl for I8 {
    type Key<'a> = i8;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::I8Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::I8Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::I8Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::I8Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_i8(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_i8()
            .map_err(|_| DeserializeError::UnexpectedEoi)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(1)
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src
            .try_get_i8()
            .map_err(|_| ValueConversionError::Deserialize(DeserializeError::UnexpectedEoi))?;

        dst.put_i8(key);
        Ok(())
    }
}

impl Sealed for U16 {}

impl KeyTagImpl for U16 {
    type Key<'a> = u16;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::U16Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::U16Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::U16Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::U16Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u16_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u16_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_u16_le()?;
        dst.put_varint_u16_le(key);
        Ok(())
    }
}

impl Sealed for I16 {}

impl KeyTagImpl for I16 {
    type Key<'a> = i16;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::I16Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::I16Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::I16Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::I16Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i16_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i16_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_i16_le()?;
        dst.put_varint_i16_le(key);
        Ok(())
    }
}

impl Sealed for U32 {}

impl KeyTagImpl for U32 {
    type Key<'a> = u32;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::U32Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::U32Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::U32Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::U32Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u32_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u32_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_u32_le()?;
        dst.put_varint_u32_le(key);
        Ok(())
    }
}

impl Sealed for I32 {}

impl KeyTagImpl for I32 {
    type Key<'a> = i32;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::I32Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::I32Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::I32Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::I32Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i32_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i32_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_i32_le()?;
        dst.put_varint_i32_le(key);
        Ok(())
    }
}

impl Sealed for U64 {}

impl KeyTagImpl for U64 {
    type Key<'a> = u64;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::U64Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::U64Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::U64Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::U64Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_u64_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_u64_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_u64_le()?;
        dst.put_varint_u64_le(key);
        Ok(())
    }
}

impl Sealed for I64 {}

impl KeyTagImpl for I64 {
    type Key<'a> = i64;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::I64Map1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::I64Map2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::I64Set1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::I64Set2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_varint_i64_le(key);
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        buf.try_get_varint_i64_le()
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip_varint_le::<{ mem::size_of::<Self>() }>()
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let key = src.try_get_varint_i64_le()?;
        dst.put_varint_i64_le(key);
        Ok(())
    }
}

impl Sealed for String {}

impl KeyTagImpl for String {
    type Key<'a> = Cow<'a, str>;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::StringMap1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::StringMap2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::StringSet1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::StringSet2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        if key.len() <= u32::MAX as usize {
            buf.put_varint_u32_le(key.len() as u32);
            buf.put_slice(key.as_bytes());
            Ok(())
        } else {
            Err(SerializeError::Overflow)
        }
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        let bytes = buf.try_copy_to_bytes(len)?.into();

        std::string::String::from_utf8(bytes)
            .map(Cow::Owned)
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        buf.try_skip(len)
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let len = src.try_get_varint_u32_le()? as usize;

        if src.len() >= len {
            dst.put_varint_u32_le(len as u32);
            dst.put_slice(&src[..len]);
            src.advance(len);

            Ok(())
        } else {
            Err(ValueConversionError::Deserialize(
                DeserializeError::UnexpectedEoi,
            ))
        }
    }
}

impl Sealed for Uuid {}

impl KeyTagImpl for Uuid {
    type Key<'a> = uuid::Uuid;

    const VALUE_KIND_MAP1: ValueKind = ValueKind::UuidMap1;
    const VALUE_KIND_MAP2: ValueKind = ValueKind::UuidMap2;
    const VALUE_KIND_SET1: ValueKind = ValueKind::UuidSet1;
    const VALUE_KIND_SET2: ValueKind = ValueKind::UuidSet2;

    fn serialize_key<B: BufMut>(key: Self::Key<'_>, buf: &mut B) -> Result<(), SerializeError> {
        buf.put_slice(key.as_bytes());
        Ok(())
    }

    fn deserialize_key<B: Buf>(buf: &mut B) -> Result<Self::Key<'_>, DeserializeError> {
        let mut bytes = uuid::Bytes::default();

        buf.try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        Ok(uuid::Uuid::from_bytes(bytes))
    }

    fn skip<B: Buf>(buf: &mut B) -> Result<(), DeserializeError> {
        buf.try_skip(16)
    }

    fn convert(src: &mut &[u8], dst: &mut BytesMut) -> Result<(), ValueConversionError> {
        let mut bytes = uuid::Bytes::default();

        src.try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        dst.put_slice(&bytes);
        Ok(())
    }
}
