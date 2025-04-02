use crate::buf_ext::{BufMutExt, ValueBufExt};
use crate::message::MessageOps;
use crate::tags::{self, KeyTagImpl};
use crate::{
    DeserializeError, ProtocolVersion, SerializeError, SerializedValue, SerializedValueSlice,
    ValueKind, MAX_VALUE_DEPTH,
};
use bytes::{Buf, BufMut, BytesMut};
use std::borrow::Cow;
use thiserror::Error;

pub(crate) fn convert(
    value: &SerializedValueSlice,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<Cow<SerializedValueSlice>, ValueConversionError> {
    const MAX: ProtocolVersion = ProtocolVersion::V1_20;

    let from = Epoch::try_from(from.unwrap_or(MAX))?;
    let to = Epoch::try_from(to)?;

    if to < from {
        let mut src = &**value;
        let mut dst = SerializedValue::new().into_bytes_mut();

        let convert = Convert::new(&mut src, &mut dst, to, 0)?;
        convert.convert()?;

        if src.is_empty() {
            Ok(Cow::Owned(SerializedValue::from_bytes_mut(dst)))
        } else {
            Err(ValueConversionError::Deserialize(
                DeserializeError::TrailingData,
            ))
        }
    } else {
        Ok(Cow::Borrowed(value))
    }
}

pub(crate) fn convert_mut(
    value: &mut SerializedValue,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<(), ValueConversionError> {
    match convert(value, from, to)? {
        Cow::Owned(converted) => {
            *value = converted;
            Ok(())
        }

        Cow::Borrowed(_) => Ok(()),
    }
}

pub(crate) fn convert_in_message(
    msg: &mut impl MessageOps,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<(), ValueConversionError> {
    match msg.value_mut() {
        Some(value) => convert_mut(value, from, to),
        None => Ok(()),
    }
}

/// Error when converting a value.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueConversionError {
    /// The requested version is invalid.
    #[error("invalid protocol version")]
    InvalidVersion,

    /// A value failed to serialize.
    #[error(transparent)]
    Serialize(#[from] SerializeError),

    /// A value failed to deserialize.
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Epoch {
    V1,
    V2,
}

impl TryFrom<ProtocolVersion> for Epoch {
    type Error = ValueConversionError;

    fn try_from(version: ProtocolVersion) -> Result<Self, Self::Error> {
        const V1_MIN: ProtocolVersion = ProtocolVersion::V1_14;
        const V1_MAX: ProtocolVersion = ProtocolVersion::V1_19;
        const V2_MIN: ProtocolVersion = ProtocolVersion::V1_20;
        const V2_MAX: ProtocolVersion = ProtocolVersion::V1_20;

        if (version >= V1_MIN) && (version <= V1_MAX) {
            Ok(Self::V1)
        } else if (version >= V2_MIN) && (version <= V2_MAX) {
            Ok(Self::V2)
        } else {
            Err(ValueConversionError::InvalidVersion)
        }
    }
}

struct Convert<'a, 'b> {
    src: &'a mut &'b [u8],
    dst: &'a mut BytesMut,
    epoch: Epoch,
    depth: u8,
}

impl<'a, 'b> Convert<'a, 'b> {
    pub(crate) fn new(
        src: &'a mut &'b [u8],
        dst: &'a mut BytesMut,
        epoch: Epoch,
        depth: u8,
    ) -> Result<Self, ValueConversionError> {
        debug_assert!(epoch < Epoch::V2);

        let mut this = Self {
            src,
            dst,
            epoch,
            depth,
        };

        this.increment_depth()?;
        Ok(this)
    }

    fn increment_depth(&mut self) -> Result<(), ValueConversionError> {
        self.depth += 1;

        if self.depth <= MAX_VALUE_DEPTH {
            Ok(())
        } else {
            Err(ValueConversionError::Deserialize(
                DeserializeError::TooDeeplyNested,
            ))
        }
    }

    fn convert(self) -> Result<(), ValueConversionError> {
        match self.src.try_get_discriminant_u8()? {
            ValueKind::None => self.convert_none(),
            ValueKind::Some => self.convert_some(),
            ValueKind::Bool => self.convert_bool(),
            ValueKind::U8 => self.convert_u8(),
            ValueKind::I8 => self.convert_i8(),
            ValueKind::U16 => self.convert_u16(),
            ValueKind::I16 => self.convert_i16(),
            ValueKind::U32 => self.convert_u32(),
            ValueKind::I32 => self.convert_i32(),
            ValueKind::U64 => self.convert_u64(),
            ValueKind::I64 => self.convert_i64(),
            ValueKind::F32 => self.convert_f32(),
            ValueKind::F64 => self.convert_f64(),
            ValueKind::String => self.convert_string(),
            ValueKind::Uuid => self.convert_uuid(),
            ValueKind::ObjectId => self.convert_object_id(),
            ValueKind::ServiceId => self.convert_service_id(),
            ValueKind::Vec1 => self.convert_vec1(),
            ValueKind::Bytes1 => self.convert_bytes1(),
            ValueKind::U8Map1 => self.convert_map1::<tags::U8>(),
            ValueKind::I8Map1 => self.convert_map1::<tags::I8>(),
            ValueKind::U16Map1 => self.convert_map1::<tags::U16>(),
            ValueKind::I16Map1 => self.convert_map1::<tags::I16>(),
            ValueKind::U32Map1 => self.convert_map1::<tags::U32>(),
            ValueKind::I32Map1 => self.convert_map1::<tags::I32>(),
            ValueKind::U64Map1 => self.convert_map1::<tags::U64>(),
            ValueKind::I64Map1 => self.convert_map1::<tags::I64>(),
            ValueKind::StringMap1 => self.convert_map1::<tags::String>(),
            ValueKind::UuidMap1 => self.convert_map1::<tags::Uuid>(),
            ValueKind::U8Set1 => self.convert_set1::<tags::U8>(),
            ValueKind::I8Set1 => self.convert_set1::<tags::I8>(),
            ValueKind::U16Set1 => self.convert_set1::<tags::U16>(),
            ValueKind::I16Set1 => self.convert_set1::<tags::I16>(),
            ValueKind::U32Set1 => self.convert_set1::<tags::U32>(),
            ValueKind::I32Set1 => self.convert_set1::<tags::I32>(),
            ValueKind::U64Set1 => self.convert_set1::<tags::U64>(),
            ValueKind::I64Set1 => self.convert_set1::<tags::I64>(),
            ValueKind::StringSet1 => self.convert_set1::<tags::String>(),
            ValueKind::UuidSet1 => self.convert_set1::<tags::Uuid>(),
            ValueKind::Struct => self.convert_struct(),
            ValueKind::Enum => self.convert_enum(),
            ValueKind::Sender => self.convert_sender(),
            ValueKind::Receiver => self.convert_receiver(),
            ValueKind::Vec2 => self.convert_vec2(),
            ValueKind::Bytes2 => self.convert_bytes2(),
            ValueKind::U8Map2 => self.convert_map2::<tags::U8>(),
            ValueKind::I8Map2 => self.convert_map2::<tags::I8>(),
            ValueKind::U16Map2 => self.convert_map2::<tags::U16>(),
            ValueKind::I16Map2 => self.convert_map2::<tags::I16>(),
            ValueKind::U32Map2 => self.convert_map2::<tags::U32>(),
            ValueKind::I32Map2 => self.convert_map2::<tags::I32>(),
            ValueKind::U64Map2 => self.convert_map2::<tags::U64>(),
            ValueKind::I64Map2 => self.convert_map2::<tags::I64>(),
            ValueKind::StringMap2 => self.convert_map2::<tags::String>(),
            ValueKind::UuidMap2 => self.convert_map2::<tags::Uuid>(),
            ValueKind::U8Set2 => self.convert_set2::<tags::U8>(),
            ValueKind::I8Set2 => self.convert_set2::<tags::I8>(),
            ValueKind::U16Set2 => self.convert_set2::<tags::U16>(),
            ValueKind::I16Set2 => self.convert_set2::<tags::I16>(),
            ValueKind::U32Set2 => self.convert_set2::<tags::U32>(),
            ValueKind::I32Set2 => self.convert_set2::<tags::I32>(),
            ValueKind::U64Set2 => self.convert_set2::<tags::U64>(),
            ValueKind::I64Set2 => self.convert_set2::<tags::I64>(),
            ValueKind::StringSet2 => self.convert_set2::<tags::String>(),
            ValueKind::UuidSet2 => self.convert_set2::<tags::Uuid>(),
        }
    }

    fn convert_next(&mut self) -> Result<(), ValueConversionError> {
        let this = Convert::new(self.src, self.dst, self.epoch, self.depth)?;
        this.convert()
    }

    fn convert_none(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::None);
        Ok(())
    }

    fn convert_some(mut self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Some);
        self.convert_next()
    }

    fn convert_bool(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Bool);

        let value = self
            .src
            .try_get_u8()
            .map(|v| v != 0)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_u8(value.into());
        Ok(())
    }

    fn convert_u8(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::U8);

        let value = self
            .src
            .try_get_u8()
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_u8(value);
        Ok(())
    }

    fn convert_i8(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::I8);

        let value = self
            .src
            .try_get_i8()
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_i8(value);
        Ok(())
    }

    fn convert_u16(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::U16);
        let value = self.src.try_get_varint_u16_le()?;
        self.dst.put_varint_u16_le(value);
        Ok(())
    }

    fn convert_i16(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::I16);
        let value = self.src.try_get_varint_i16_le()?;
        self.dst.put_varint_i16_le(value);
        Ok(())
    }

    fn convert_u32(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::U32);
        let value = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(value);
        Ok(())
    }

    fn convert_i32(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::I32);
        let value = self.src.try_get_varint_i32_le()?;
        self.dst.put_varint_i32_le(value);
        Ok(())
    }

    fn convert_u64(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::U64);
        let value = self.src.try_get_varint_u64_le()?;
        self.dst.put_varint_u64_le(value);
        Ok(())
    }

    fn convert_i64(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::I64);
        let value = self.src.try_get_varint_i64_le()?;
        self.dst.put_varint_i64_le(value);
        Ok(())
    }

    fn convert_f32(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::F32);

        let value = self
            .src
            .try_get_f32_le()
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_u32_le(value.to_bits());
        Ok(())
    }

    fn convert_f64(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::F64);

        let value = self
            .src
            .try_get_f64_le()
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_u64_le(value.to_bits());
        Ok(())
    }

    fn convert_string(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::String);
        let len = self.src.try_get_varint_u32_le()? as usize;

        if self.src.len() >= len {
            self.dst.put_varint_u32_le(len as u32);
            self.dst.put_slice(&self.src[..len]);
            self.src.advance(len);

            Ok(())
        } else {
            Err(ValueConversionError::Deserialize(
                DeserializeError::UnexpectedEoi,
            ))
        }
    }

    fn convert_uuid(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Uuid);
        let mut bytes = uuid::Bytes::default();

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_slice(&bytes);
        Ok(())
    }

    fn convert_object_id(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::ObjectId);
        let mut bytes = uuid::Bytes::default();

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        Ok(())
    }

    fn convert_service_id(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::ServiceId);
        let mut bytes = uuid::Bytes::default();

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;
        self.dst.put_slice(&bytes);

        Ok(())
    }

    fn convert_vec1(mut self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Vec1);

        let len = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(len);

        for _ in 0..len {
            self.convert_next()?;
        }

        Ok(())
    }

    fn convert_bytes1(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Bytes1);
        let len = self.src.try_get_varint_u32_le()? as usize;

        if self.src.len() >= len {
            self.dst.put_varint_u32_le(len as u32);
            self.dst.put_slice(&self.src[..len]);
            self.src.advance(len);

            Ok(())
        } else {
            Err(ValueConversionError::Deserialize(
                DeserializeError::UnexpectedEoi,
            ))
        }
    }

    fn convert_map1<K: KeyTagImpl>(mut self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(K::VALUE_KIND_MAP1);

        let len = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(len);

        for _ in 0..len {
            K::convert(self.src, self.dst)?;
            self.convert_next()?;
        }

        Ok(())
    }

    fn convert_set1<K: KeyTagImpl>(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(K::VALUE_KIND_SET1);

        let len = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(len);

        for _ in 0..len {
            K::convert(self.src, self.dst)?;
        }

        Ok(())
    }

    fn convert_struct(mut self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Struct);

        let len = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(len);

        for _ in 0..len {
            let id = self.src.try_get_varint_u32_le()?;
            self.dst.put_varint_u32_le(id);

            self.convert_next()?;
        }

        Ok(())
    }

    fn convert_enum(mut self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Enum);

        let variant = self.src.try_get_varint_u32_le()?;
        self.dst.put_varint_u32_le(variant);

        self.convert_next()
    }

    fn convert_sender(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Sender);
        let mut bytes = uuid::Bytes::default();

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_slice(&bytes);
        Ok(())
    }

    fn convert_receiver(self) -> Result<(), ValueConversionError> {
        self.dst.put_discriminant_u8(ValueKind::Receiver);
        let mut bytes = uuid::Bytes::default();

        self.src
            .try_copy_to_slice(&mut bytes)
            .map_err(|_| DeserializeError::UnexpectedEoi)?;

        self.dst.put_slice(&bytes);
        Ok(())
    }

    fn convert_vec2(self) -> Result<(), ValueConversionError> {
        match self.epoch {
            Epoch::V1 => self.convert_vec2_to_vec1(),
            Epoch::V2 => unreachable!(),
        }
    }

    fn convert_vec2_to_vec1(self) -> Result<(), ValueConversionError> {
        let mut tmp = BytesMut::new();
        let mut len = 0usize;

        loop {
            match self.src.try_get_discriminant_u8()? {
                ValueKind::None => {
                    let Ok(len) = len.try_into() else {
                        return Err(ValueConversionError::Serialize(SerializeError::Overflow));
                    };

                    self.dst.put_discriminant_u8(ValueKind::Vec1);
                    self.dst.put_varint_u32_le(len);
                    self.dst.extend_from_slice(&tmp);

                    break Ok(());
                }

                ValueKind::Some => {
                    let convert = Convert::new(self.src, &mut tmp, self.epoch, self.depth)?;
                    convert.convert()?;

                    len += 1;
                }

                _ => {
                    break Err(ValueConversionError::Deserialize(
                        DeserializeError::InvalidSerialization,
                    ));
                }
            }
        }
    }

    fn convert_bytes2(self) -> Result<(), ValueConversionError> {
        match self.epoch {
            Epoch::V1 => self.convert_bytes2_to_bytes1(),
            Epoch::V2 => unreachable!(),
        }
    }

    fn convert_bytes2_to_bytes1(self) -> Result<(), ValueConversionError> {
        let mut tmp = Vec::new();

        loop {
            let len = self.src.try_get_varint_u32_le()? as usize;

            if len == 0 {
                let Ok(len) = tmp.len().try_into() else {
                    return Err(ValueConversionError::Serialize(SerializeError::Overflow));
                };

                self.dst.put_discriminant_u8(ValueKind::Bytes1);
                self.dst.put_varint_u32_le(len);
                self.dst.extend_from_slice(&tmp);

                break Ok(());
            }

            if self.src.len() < len {
                break Err(ValueConversionError::Deserialize(
                    DeserializeError::InvalidSerialization,
                ));
            }

            tmp.extend_from_slice(&self.src[..len]);
            self.src.advance(len);
        }
    }

    fn convert_map2<K: KeyTagImpl>(self) -> Result<(), ValueConversionError> {
        match self.epoch {
            Epoch::V1 => self.convert_map2_to_map1::<K>(),
            Epoch::V2 => unreachable!(),
        }
    }

    fn convert_map2_to_map1<K: KeyTagImpl>(self) -> Result<(), ValueConversionError> {
        let mut tmp = BytesMut::new();
        let mut len = 0usize;

        loop {
            match self.src.try_get_discriminant_u8()? {
                ValueKind::None => {
                    let Ok(len) = len.try_into() else {
                        return Err(ValueConversionError::Serialize(SerializeError::Overflow));
                    };

                    self.dst.put_discriminant_u8(K::VALUE_KIND_MAP1);
                    self.dst.put_varint_u32_le(len);
                    self.dst.extend_from_slice(&tmp);

                    break Ok(());
                }

                ValueKind::Some => {
                    K::convert(self.src, &mut tmp)?;

                    let convert = Convert::new(self.src, &mut tmp, self.epoch, self.depth)?;
                    convert.convert()?;

                    len += 1;
                }

                _ => {
                    break Err(ValueConversionError::Deserialize(
                        DeserializeError::InvalidSerialization,
                    ));
                }
            }
        }
    }

    fn convert_set2<K: KeyTagImpl>(self) -> Result<(), ValueConversionError> {
        match self.epoch {
            Epoch::V1 => self.convert_set2_to_set1::<K>(),
            Epoch::V2 => unreachable!(),
        }
    }

    fn convert_set2_to_set1<K: KeyTagImpl>(self) -> Result<(), ValueConversionError> {
        let mut tmp = BytesMut::new();
        let mut len = 0usize;

        loop {
            match self.src.try_get_discriminant_u8()? {
                ValueKind::None => {
                    let Ok(len) = len.try_into() else {
                        return Err(ValueConversionError::Serialize(SerializeError::Overflow));
                    };

                    self.dst.put_discriminant_u8(K::VALUE_KIND_SET1);
                    self.dst.put_varint_u32_le(len);
                    self.dst.extend_from_slice(&tmp);

                    break Ok(());
                }

                ValueKind::Some => {
                    K::convert(self.src, &mut tmp)?;
                    len += 1;
                }

                _ => {
                    break Err(ValueConversionError::Deserialize(
                        DeserializeError::InvalidSerialization,
                    ));
                }
            }
        }
    }
}
