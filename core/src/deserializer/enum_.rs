use super::Deserializer;
use crate::buf_ext::ValueBufExt;
use crate::tags::{self, Tag};
use crate::{Deserialize, DeserializeError, UnknownVariant, ValueKind};

#[derive(Debug)]
pub struct EnumDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    variant: u32,
    depth: u8,
}

impl<'a, 'b> EnumDeserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Enum)?;
        Self::new_without_value_kind(buf, depth)
    }

    pub(super) fn new_without_value_kind(
        buf: &'a mut &'b [u8],
        depth: u8,
    ) -> Result<Self, DeserializeError> {
        let variant = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            variant,
            depth,
        })
    }

    pub fn variant(&self) -> u32 {
        self.variant
    }

    pub fn try_variant<T: TryFrom<u32>>(&self) -> Result<T, DeserializeError> {
        self.variant
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<U, DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.deserialize()
    }

    pub fn deserialize_unit(self) -> Result<(), DeserializeError> {
        self.deserialize::<tags::Unit, _>()
    }

    pub fn into_unknown_variant(self) -> Result<UnknownVariant, DeserializeError> {
        let id = self.variant;
        let value = self.deserialize()?;
        Ok(UnknownVariant::new(id, value))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }
}
