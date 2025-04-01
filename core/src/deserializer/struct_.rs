use super::Deserializer;
use crate::buf_ext::ValueBufExt;
use crate::tags::Tag;
use crate::{Deserialize, DeserializeError, UnknownFields, ValueKind};

#[derive(Debug)]
pub struct StructDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    unknown_fields: UnknownFields,
}

impl<'a, 'b> StructDeserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Struct)?;
        Self::new_without_value_kind(buf, depth)
    }

    pub(super) fn new_without_value_kind(
        buf: &'a mut &'b [u8],
        depth: u8,
    ) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            depth,
            unknown_fields: UnknownFields::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize(&mut self) -> Result<FieldDeserializer<'_, 'b>, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            FieldDeserializer::new(self.buf, self.depth, &mut self.unknown_fields)
        }
    }

    pub fn deserialize_specific_field<T: Tag, U: Deserialize<T>>(
        &mut self,
        id: impl Into<u32>,
    ) -> Result<U, DeserializeError> {
        let field = self.deserialize()?;

        if field.id() == id.into() {
            field.deserialize()
        } else {
            Err(DeserializeError::InvalidSerialization)
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.deserialize()?.skip()?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|_| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f(self.unknown_fields)
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|_| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(mut self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce(UnknownFields) -> Result<T, DeserializeError>,
    {
        while !self.is_empty() {
            self.deserialize()?.skip()?;
        }

        f(self.unknown_fields)
    }
}

#[derive(Debug)]
pub struct FieldDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    id: u32,
    depth: u8,
    unknown_fields: &'a mut UnknownFields,
}

impl<'a, 'b> FieldDeserializer<'a, 'b> {
    fn new(
        buf: &'a mut &'b [u8],
        depth: u8,
        unknown_fields: &'a mut UnknownFields,
    ) -> Result<Self, DeserializeError> {
        let id = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            id,
            depth,
            unknown_fields,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn try_id<T: TryFrom<u32>>(&self) -> Result<T, DeserializeError> {
        self.id
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<U, DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.deserialize()
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }

    pub fn add_to_unknown_fields(self) -> Result<(), DeserializeError> {
        let deserializer = Deserializer::new(self.buf, self.depth)?;
        let value = deserializer.deserialize()?;
        self.unknown_fields.insert(self.id, value);
        Ok(())
    }
}
