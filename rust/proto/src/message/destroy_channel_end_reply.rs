use super::message_ops::Sealed;
use super::{MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DestroyChannelEndResult {
    Ok = 0,
    InvalidChannel = 1,
    ForeignChannel = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DestroyChannelEndReply {
    pub serial: u32,
    pub result: DestroyChannelEndResult,
}

impl MessageOps for DestroyChannelEndReply {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyChannelEndReply
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyChannelEndReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyChannelEndReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for DestroyChannelEndReply {}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{DestroyChannelEndReply, DestroyChannelEndResult};

    #[test]
    fn ok() {
        let serialized = [34, 1, 0];

        let msg = DestroyChannelEndReply {
            serial: 1,
            result: DestroyChannelEndResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_channel() {
        let serialized = [34, 1, 1];

        let msg = DestroyChannelEndReply {
            serial: 1,
            result: DestroyChannelEndResult::InvalidChannel,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn foreign_channel() {
        let serialized = [34, 1, 2];

        let msg = DestroyChannelEndReply {
            serial: 1,
            result: DestroyChannelEndResult::ForeignChannel,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
