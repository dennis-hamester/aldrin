use crate::buf_ext::BufMutExt;
use crate::message::MessageKind;
use crate::serialized_value::SerializedValue;
use bytes::{BufMut, BytesMut};
use thiserror::Error;
use uuid::Uuid;

pub(crate) struct MessageSerializer {
    buf: BytesMut,
}

impl MessageSerializer {
    pub fn without_value(kind: MessageKind) -> Self {
        debug_assert!(!kind.has_value());

        let mut buf = BytesMut::zeroed(4);
        buf.put_u8(kind.into());

        Self { buf }
    }

    pub fn with_value(
        value: SerializedValue,
        kind: MessageKind,
    ) -> Result<Self, MessageSerializeError> {
        debug_assert!(kind.has_value());

        let mut buf = value.into_bytes_mut();

        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        if buf.len() < 10 {
            return Err(MessageSerializeError::InvalidValue);
        }

        let value_len = buf.len() - 9;
        if value_len > u32::MAX as usize {
            return Err(MessageSerializeError::Overflow);
        }

        buf[4] = kind.into();
        buf[5..9].copy_from_slice(&(value_len as u32).to_le_bytes());

        Ok(Self { buf })
    }

    pub fn with_none_value(kind: MessageKind) -> Self {
        Self::with_value(SerializedValue::serialize(&()).unwrap(), kind).unwrap()
    }

    pub fn put_discriminant_u8(&mut self, discriminant: impl Into<u8>) {
        self.buf.put_discriminant_u8(discriminant);
    }

    pub fn put_varint_u32_le(&mut self, n: u32) {
        self.buf.put_varint_u32_le(n);
    }

    pub fn put_uuid(&mut self, uuid: Uuid) {
        self.buf.put_slice(uuid.as_ref());
    }

    pub fn finish(mut self) -> Result<BytesMut, MessageSerializeError> {
        let len = self.buf.len();
        if len <= u32::MAX as usize {
            self.buf[..4].copy_from_slice(&(len as u32).to_le_bytes());
            Ok(self.buf)
        } else {
            Err(MessageSerializeError::Overflow)
        }
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageSerializeError {
    #[error("serialized message overflowed")]
    Overflow,

    #[error("invalid value")]
    InvalidValue,
}
