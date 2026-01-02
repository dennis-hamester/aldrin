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
        let num_elems = u32::try_from(num_elems).map_err(|_| SerializeError::Overflow)?;

        buf.put_discriminant_u8(ValueKind::Bytes1);
        buf.put_varint_u32_le(num_elems);

        Ok(Self { buf, num_elems })
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
    pub(super) fn new(buf: &'a mut BytesMut) -> Self {
        buf.put_discriminant_u8(ValueKind::Bytes2);
        Self { buf }
    }

    pub fn serialize(&mut self, bytes: &[u8]) -> Result<&mut Self, SerializeError> {
        if bytes.is_empty() {
            Ok(self)
        } else if let Ok(len) = u32::try_from(bytes.len()) {
            self.buf.put_varint_u32_le(len);
            self.buf.put_slice(bytes);
            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_varint_u32_le(0);
        Ok(())
    }
}
