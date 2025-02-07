use super::{MessageDeserializeError, MessageKind};
use crate::buf_ext::MessageBufExt;
use crate::SerializedValue;
use bytes::{Buf, BytesMut};
use uuid::Uuid;

pub(crate) struct MessageWithoutValueDeserializer {
    buf: BytesMut,
}

impl MessageWithoutValueDeserializer {
    pub fn new(mut buf: BytesMut, kind: MessageKind) -> Result<Self, MessageDeserializeError> {
        let buf_len = buf.len();

        // 4 bytes message length + 1 byte message kind.
        if buf_len < 5 {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let len = buf.get_u32_le() as usize;
        if buf_len != len {
            return Err(MessageDeserializeError::InvalidSerialization);
        }

        buf.ensure_discriminant_u8(kind)?;

        Ok(Self { buf })
    }

    pub fn try_get_discriminant_u8<T: TryFrom<u8>>(
        &mut self,
    ) -> Result<T, MessageDeserializeError> {
        self.buf.try_get_discriminant_u8()
    }

    pub fn try_get_varint_u32_le(&mut self) -> Result<u32, MessageDeserializeError> {
        self.buf.try_get_varint_u32_le()
    }

    pub fn try_get_uuid(&mut self) -> Result<Uuid, MessageDeserializeError> {
        let mut bytes = uuid::Bytes::default();
        self.buf.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn finish(self) -> Result<(), MessageDeserializeError> {
        if self.buf.is_empty() {
            Ok(())
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }
}

pub(crate) struct MessageWithValueDeserializer {
    header_and_value: BytesMut,
    msg: BytesMut,
}

impl MessageWithValueDeserializer {
    pub fn new(mut buf: BytesMut, kind: MessageKind) -> Result<Self, MessageDeserializeError> {
        debug_assert!(kind.has_value());

        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        if buf.len() < 10 {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let msg_len = (&buf[..4]).get_u32_le() as usize;
        if buf.len() != msg_len {
            return Err(MessageDeserializeError::InvalidSerialization);
        }

        if buf[4] != kind.into() {
            return Err(MessageDeserializeError::UnexpectedMessage);
        }

        let value_len = (&buf[5..9]).get_u32_le() as usize;
        let max_value_len = buf.len() - 9;

        if value_len < 1 {
            return Err(MessageDeserializeError::InvalidSerialization);
        } else if value_len > max_value_len {
            return Err(MessageDeserializeError::UnexpectedEoi);
        }

        let msg = buf.split_off(9 + value_len);
        Ok(Self {
            header_and_value: buf,
            msg,
        })
    }

    pub fn try_get_discriminant_u8<T: TryFrom<u8>>(
        &mut self,
    ) -> Result<T, MessageDeserializeError> {
        self.msg.try_get_discriminant_u8()
    }

    pub fn try_get_varint_u32_le(&mut self) -> Result<u32, MessageDeserializeError> {
        self.msg.try_get_varint_u32_le()
    }

    pub fn try_get_uuid(&mut self) -> Result<Uuid, MessageDeserializeError> {
        let mut bytes = uuid::Bytes::default();
        self.msg.try_copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn finish(mut self) -> Result<SerializedValue, MessageDeserializeError> {
        if self.msg.is_empty() {
            self.header_and_value.unsplit(self.msg);
            self.header_and_value[0..9].fill(0);
            Ok(SerializedValue::from_bytes_mut(self.header_and_value))
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }

    pub fn finish_discard_value(self) -> Result<(), MessageDeserializeError> {
        if self.msg.is_empty() {
            Ok(())
        } else {
            Err(MessageDeserializeError::TrailingData)
        }
    }
}
