use crate::buf_ext::BufMutExt;
use crate::{SerializeError, ValueKind};
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct Bytes1Serializer<'a> {
    buf: &'a mut BytesMut,
    num_elems: u32,
}

impl<'a> Bytes1Serializer<'a> {
    pub(super) fn new(buf: &'a mut BytesMut, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.put_discriminant_u8(ValueKind::Bytes1);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
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

    pub fn serialize(&mut self, bytes: &[u8]) -> Result<&mut Self, SerializeError> {
        if bytes.len() <= self.num_elems as usize {
            self.num_elems -= bytes.len() as u32;
            self.buf.put_slice(bytes);
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
pub struct Bytes2Serializer<'a> {
    buf: &'a mut BytesMut,
}

impl<'a> Bytes2Serializer<'a> {
    pub(super) fn new(buf: &'a mut BytesMut) -> Result<Self, SerializeError> {
        buf.put_discriminant_u8(ValueKind::Bytes2);
        Ok(Self { buf })
    }

    pub fn serialize(&mut self, bytes: &[u8]) -> Result<&mut Self, SerializeError> {
        if !bytes.is_empty() {
            if bytes.len() <= u32::MAX as usize {
                self.buf.put_varint_u32_le(bytes.len() as u32);
                self.buf.put_slice(bytes);
                Ok(self)
            } else {
                Err(SerializeError::TooManyElements)
            }
        } else {
            Ok(self)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_varint_u32_le(0);
        Ok(())
    }
}
