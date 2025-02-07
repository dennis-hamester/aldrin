#[cfg(test)]
mod test;

use crate::error::DeserializeError;
use crate::message_deserializer::MessageDeserializeError;
use bytes::{Buf, BufMut, Bytes};

pub(crate) trait BufMutExt: BufMut {
    fn put_discriminant_u8(&mut self, discriminant: impl Into<u8>) {
        self.put_u8(discriminant.into())
    }

    fn put_varint_u16_le(&mut self, n: u16) {
        self.put_varint_le(n.to_le_bytes());
    }

    fn put_varint_i16_le(&mut self, n: i16) {
        self.put_varint_u16_le(zigzag_encode_i16(n));
    }

    fn put_varint_u32_le(&mut self, n: u32) {
        self.put_varint_le(n.to_le_bytes());
    }

    fn put_varint_i32_le(&mut self, n: i32) {
        self.put_varint_u32_le(zigzag_encode_i32(n));
    }

    fn put_varint_u64_le(&mut self, n: u64) {
        self.put_varint_le(n.to_le_bytes());
    }

    fn put_varint_i64_le(&mut self, n: i64) {
        self.put_varint_u64_le(zigzag_encode_i64(n));
    }

    fn put_varint_le<const N: usize>(&mut self, bytes: [u8; N]) {
        for (i, n) in bytes.into_iter().rev().enumerate().take(N - 1) {
            if n != 0 {
                self.put_u8(255 - i as u8);
                self.put_slice(&bytes[..N - i]);
                return;
            }
        }

        if bytes[0] > 255 - N as u8 {
            self.put_u8(255 - N as u8 + 1);
        }

        self.put_u8(bytes[0]);
    }
}

impl<T: BufMut + ?Sized> BufMutExt for T {}

pub(crate) trait ValueBufExt: Buf {
    fn try_get_discriminant_u8<T: TryFrom<u8>>(&mut self) -> Result<T, DeserializeError> {
        self.try_get_u8()?
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)
    }

    fn try_peek_discriminant_u8<T: TryFrom<u8>>(&self) -> Result<T, DeserializeError> {
        if self.remaining() >= 1 {
            self.chunk()[0]
                .try_into()
                .map_err(|_| DeserializeError::InvalidSerialization)
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn ensure_discriminant_u8<T: TryFrom<u8> + PartialEq>(
        &mut self,
        discriminant: T,
    ) -> Result<(), DeserializeError> {
        if self.try_get_discriminant_u8::<T>()? == discriminant {
            Ok(())
        } else {
            Err(DeserializeError::UnexpectedValue)
        }
    }

    fn try_get_u8(&mut self) -> Result<u8, DeserializeError> {
        if self.remaining() >= 1 {
            Ok(self.get_u8())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_get_i8(&mut self) -> Result<i8, DeserializeError> {
        if self.remaining() >= 1 {
            Ok(self.get_i8())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_get_u32_le(&mut self) -> Result<u32, DeserializeError> {
        if self.remaining() >= 4 {
            Ok(self.get_u32_le())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_get_u64_le(&mut self) -> Result<u64, DeserializeError> {
        if self.remaining() >= 8 {
            Ok(self.get_u64_le())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_get_varint_u16_le(&mut self) -> Result<u16, DeserializeError> {
        self.try_get_varint_le().map(u16::from_le_bytes)
    }

    fn try_get_varint_i16_le(&mut self) -> Result<i16, DeserializeError> {
        self.try_get_varint_u16_le().map(zigzag_decode_i16)
    }

    fn try_get_varint_u32_le(&mut self) -> Result<u32, DeserializeError> {
        self.try_get_varint_le().map(u32::from_le_bytes)
    }

    fn try_get_varint_i32_le(&mut self) -> Result<i32, DeserializeError> {
        self.try_get_varint_u32_le().map(zigzag_decode_i32)
    }

    fn try_get_varint_u64_le(&mut self) -> Result<u64, DeserializeError> {
        self.try_get_varint_le().map(u64::from_le_bytes)
    }

    fn try_get_varint_i64_le(&mut self) -> Result<i64, DeserializeError> {
        self.try_get_varint_u64_le().map(zigzag_decode_i64)
    }

    fn try_get_varint_le<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
        let mut bytes = [0; N];
        let first = self.try_get_u8()?;

        if first > 255 - N as u8 {
            let num_bytes = first as usize + N - 255;
            self.try_copy_to_slice(&mut bytes[..num_bytes])?;
        } else {
            bytes[0] = first;
        }

        Ok(bytes)
    }

    fn try_copy_to_bytes(&mut self, len: usize) -> Result<Bytes, DeserializeError> {
        if self.remaining() >= len {
            Ok(self.copy_to_bytes(len))
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_copy_to_slice(&mut self, dst: &mut [u8]) -> Result<(), DeserializeError> {
        if self.remaining() >= dst.len() {
            self.copy_to_slice(dst);
            Ok(())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_skip(&mut self, len: usize) -> Result<(), DeserializeError> {
        if self.remaining() >= len {
            self.advance(len);
            Ok(())
        } else {
            Err(DeserializeError::UnexpectedEoi)
        }
    }

    fn try_skip_varint_le<const N: usize>(&mut self) -> Result<(), DeserializeError> {
        let first = self.try_get_u8()?;

        if first > 255 - N as u8 {
            let num_bytes = first as usize + N - 255;
            self.try_skip(num_bytes)?;
        }

        Ok(())
    }
}

impl<T: Buf + ?Sized> ValueBufExt for T {}

pub(crate) trait MessageBufExt: Buf {
    fn try_get_discriminant_u8<T: TryFrom<u8>>(&mut self) -> Result<T, MessageDeserializeError> {
        self.try_get_u8()?
            .try_into()
            .map_err(|_| MessageDeserializeError::InvalidSerialization)
    }

    fn ensure_discriminant_u8<T: TryFrom<u8> + PartialEq>(
        &mut self,
        discriminant: T,
    ) -> Result<(), MessageDeserializeError> {
        if self.try_get_discriminant_u8::<T>()? == discriminant {
            Ok(())
        } else {
            Err(MessageDeserializeError::UnexpectedMessage)
        }
    }

    fn try_get_u8(&mut self) -> Result<u8, MessageDeserializeError> {
        if self.remaining() >= 1 {
            Ok(self.get_u8())
        } else {
            Err(MessageDeserializeError::UnexpectedEoi)
        }
    }

    fn try_get_varint_u32_le(&mut self) -> Result<u32, MessageDeserializeError> {
        self.try_get_varint_le().map(u32::from_le_bytes)
    }

    fn try_get_varint_le<const N: usize>(&mut self) -> Result<[u8; N], MessageDeserializeError> {
        let mut bytes = [0; N];
        let first = self.try_get_u8()?;

        if first > 255 - N as u8 {
            let num_bytes = first as usize + N - 255;
            self.try_copy_to_slice(&mut bytes[..num_bytes])?;
        } else {
            bytes[0] = first;
        }

        Ok(bytes)
    }

    fn try_copy_to_slice(&mut self, dst: &mut [u8]) -> Result<(), MessageDeserializeError> {
        if self.remaining() >= dst.len() {
            self.copy_to_slice(dst);
            Ok(())
        } else {
            Err(MessageDeserializeError::UnexpectedEoi)
        }
    }
}

impl<T: Buf + ?Sized> MessageBufExt for T {}

fn zigzag_encode_i16(n: i16) -> u16 {
    (n >> 15) as u16 ^ (n << 1) as u16
}

fn zigzag_decode_i16(n: u16) -> i16 {
    (n >> 1) as i16 ^ -((n & 1) as i16)
}

fn zigzag_encode_i32(n: i32) -> u32 {
    (n >> 31) as u32 ^ (n << 1) as u32
}

fn zigzag_decode_i32(n: u32) -> i32 {
    (n >> 1) as i32 ^ -((n & 1) as i32)
}

fn zigzag_encode_i64(n: i64) -> u64 {
    (n >> 63) as u64 ^ (n << 1) as u64
}

fn zigzag_decode_i64(n: u64) -> i64 {
    (n >> 1) as i64 ^ -((n & 1) as i64)
}
