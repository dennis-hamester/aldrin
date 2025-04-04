use super::Serializer;
use crate::buf_ext::BufMutExt;
use crate::tags::Tag;
use crate::{AsUnknownFields, Serialize, SerializeError, UnknownFieldsRef, ValueKind};
use bytes::BytesMut;

#[derive(Debug)]
pub struct Struct1Serializer<'a> {
    buf: &'a mut BytesMut,
    num_fields: u32,
    depth: u8,
}

impl<'a> Struct1Serializer<'a> {
    pub(super) fn new(
        buf: &'a mut BytesMut,
        num_fields: usize,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        if num_fields <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Struct1);
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

    pub(super) fn with_unknown_fields<T>(
        buf: &'a mut BytesMut,
        num_fields: usize,
        unknown_fields: T,
        depth: u8,
    ) -> Result<Self, SerializeError>
    where
        T: AsUnknownFields,
        T::FieldsIter: ExactSizeIterator,
    {
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

    pub fn serialize_if_some<T: Tag, U: Serialize<T>>(
        &mut self,
        id: impl Into<u32>,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if value.serializes_as_some() {
            self.serialize(id, value)
        } else {
            Ok(self)
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

#[derive(Debug)]
pub struct Struct2Serializer<'a> {
    buf: &'a mut BytesMut,
    depth: u8,
}

impl<'a> Struct2Serializer<'a> {
    pub(super) fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        buf.put_discriminant_u8(ValueKind::Struct2);
        Ok(Self { buf, depth })
    }

    pub(super) fn with_unknown_fields(
        buf: &'a mut BytesMut,
        unknown_fields: impl AsUnknownFields,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        let mut this = Self::new(buf, depth)?;
        this.serialize_unknown_fields(unknown_fields)?;
        Ok(this)
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        id: impl Into<u32>,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Some);
        self.buf.put_varint_u32_le(id.into());

        let serializer = Serializer::new(self.buf, self.depth)?;
        serializer.serialize(value)?;

        Ok(self)
    }

    pub fn serialize_if_some<T: Tag, U: Serialize<T>>(
        &mut self,
        id: impl Into<u32>,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if value.serializes_as_some() {
            self.serialize(id, value)
        } else {
            Ok(self)
        }
    }

    pub fn serialize_unknown_fields(
        &mut self,
        unknown_fields: impl AsUnknownFields,
    ) -> Result<&mut Self, SerializeError> {
        for (id, value) in unknown_fields.fields() {
            self.serialize(id, value)?;
        }

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }
}
