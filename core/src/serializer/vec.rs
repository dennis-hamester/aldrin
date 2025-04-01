use super::Serializer;
use crate::buf_ext::BufMutExt;
use crate::tags::Tag;
use crate::{Serialize, SerializeError, ValueKind};
use bytes::BytesMut;

#[derive(Debug)]
pub struct Vec1Serializer<'a> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    depth: u8,
}

impl<'a> Vec1Serializer<'a> {
    pub(super) fn new(
        buf: &'a mut BytesMut,
        num_elems: usize,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Vec1);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
                depth,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

#[derive(Debug)]
pub struct Vec2Serializer<'a> {
    buf: &'a mut BytesMut,
    depth: u8,
}

impl<'a> Vec2Serializer<'a> {
    pub(super) fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        buf.put_discriminant_u8(ValueKind::Vec2);
        Ok(Self { buf, depth })
    }

    pub fn serialize<T: Tag, U: Serialize<T>>(
        &mut self,
        value: U,
    ) -> Result<&mut Self, SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Some);

        let serializer = Serializer::new(self.buf, self.depth)?;
        serializer.serialize(value)?;

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }
}
