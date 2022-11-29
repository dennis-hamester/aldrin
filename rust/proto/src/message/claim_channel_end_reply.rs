use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ClaimChannelEndResult {
    Ok = 0,
    InvalidChannel = 1,
    AlreadyClaimed = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClaimChannelEndReply {
    pub serial: u32,
    pub result: ClaimChannelEndResult,
}

impl MessageOps for ClaimChannelEndReply {
    fn kind(&self) -> MessageKind {
        MessageKind::ClaimChannelEndReply
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ClaimChannelEndReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ClaimChannelEndReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for ClaimChannelEndReply {}

impl From<ClaimChannelEndReply> for Message {
    fn from(msg: ClaimChannelEndReply) -> Self {
        Self::ClaimChannelEndReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{ClaimChannelEndReply, ClaimChannelEndResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 37, 1, 0];

        let msg = ClaimChannelEndReply {
            serial: 1,
            result: ClaimChannelEndResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_channel() {
        let serialized = [7, 0, 0, 0, 37, 1, 1];

        let msg = ClaimChannelEndReply {
            serial: 1,
            result: ClaimChannelEndResult::InvalidChannel,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn already_claimed() {
        let serialized = [7, 0, 0, 0, 37, 1, 2];

        let msg = ClaimChannelEndReply {
            serial: 1,
            result: ClaimChannelEndResult::AlreadyClaimed,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
