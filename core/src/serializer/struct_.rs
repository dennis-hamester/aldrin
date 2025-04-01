use super::Serializer;
use crate::buf_ext::BufMutExt;
use crate::tags::Tag;
use crate::{AsUnknownFields, Serialize, SerializeError, UnknownFieldsRef, ValueKind};
use bytes::BytesMut;

#[derive(Debug)]
pub struct StructSerializer<'a> {
    buf: &'a mut BytesMut,
    num_fields: u32,
    depth: u8,
}

impl<'a> StructSerializer<'a> {
    pub(super) fn new(
        buf: &'a mut BytesMut,
        num_fields: usize,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        if num_fields <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Struct);
            buf.put_varint_u32_le(num_fields as u32);

            Ok(Self {
                buf,
                num_fields: num_fields as u32,
                depth,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub(super) fn with_unknown_fields(
        buf: &'a mut BytesMut,
        num_fields: usize,
        unknown_fields: impl AsUnknownFields,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        let unknown_fields = unknown_fields.fields();
        let mut this = Self::new(buf, num_fields + unknown_fields.len(), depth)?;
        this.serialize_unknown_fields(UnknownFieldsRef(unknown_fields))?;
        Ok(this)
    }

    pub fn remaining_fields(&self) -> usize {
        self.num_fields as usize
    }

    pub fn requires_additional_fields(&self) -> bool {
        self.num_fields > 0
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        id: impl Into<u32>,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_fields > 0 {
            self.num_fields -= 1;

            self.buf.put_varint_u32_le(id.into());

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn serialize_unknown_fields(
        &mut self,
        unknown_fields: impl AsUnknownFields,
    ) -> Result<&mut Self, SerializeError> {
        for (id, value) in unknown_fields.fields() {
            if self.num_fields == 0 {
                return Err(SerializeError::TooManyElements);
            }

            self.num_fields -= 1;
            self.buf.put_varint_u32_le(id);

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;
        }

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_fields == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}
