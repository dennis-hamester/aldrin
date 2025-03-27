use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum ClaimChannelEndReplyKind {
    SenderClaimed = 0,
    ReceiverClaimed = 1,
    InvalidChannel = 2,
    AlreadyClaimed = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum ClaimChannelEndResult {
    SenderClaimed(u32),
    ReceiverClaimed,
    InvalidChannel,
    AlreadyClaimed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ClaimChannelEndReply {
    pub serial: u32,
    pub result: ClaimChannelEndResult,
}

impl MessageOps for ClaimChannelEndReply {
    fn kind(&self) -> MessageKind {
        MessageKind::ClaimChannelEndReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ClaimChannelEndReply);

        serializer.put_varint_u32_le(self.serial);

        match self.result {
            ClaimChannelEndResult::SenderClaimed(capacity) => {
                serializer.put_discriminant_u8(ClaimChannelEndReplyKind::SenderClaimed);
                serializer.put_varint_u32_le(capacity);
            }
            ClaimChannelEndResult::ReceiverClaimed => {
                serializer.put_discriminant_u8(ClaimChannelEndReplyKind::ReceiverClaimed)
            }
            ClaimChannelEndResult::InvalidChannel => {
                serializer.put_discriminant_u8(ClaimChannelEndReplyKind::InvalidChannel)
            }
            ClaimChannelEndResult::AlreadyClaimed => {
                serializer.put_discriminant_u8(ClaimChannelEndReplyKind::AlreadyClaimed)
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ClaimChannelEndReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let result = match deserializer.try_get_discriminant_u8()? {
            ClaimChannelEndReplyKind::SenderClaimed => {
                let capacity = deserializer.try_get_varint_u32_le()?;
                ClaimChannelEndResult::SenderClaimed(capacity)
            }
            ClaimChannelEndReplyKind::ReceiverClaimed => ClaimChannelEndResult::ReceiverClaimed,
            ClaimChannelEndReplyKind::InvalidChannel => ClaimChannelEndResult::InvalidChannel,
            ClaimChannelEndReplyKind::AlreadyClaimed => ClaimChannelEndResult::AlreadyClaimed,
        };

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
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
    fn sender_claimed() {
        let serialized = [8, 0, 0, 0, 25, 1, 0, 2];

        let msg = ClaimChannelEndReply {
            serial: 1,
            result: ClaimChannelEndResult::SenderClaimed(2),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver_claimed() {
        let serialized = [7, 0, 0, 0, 25, 1, 1];

        let msg = ClaimChannelEndReply {
            serial: 1,
            result: ClaimChannelEndResult::ReceiverClaimed,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_channel() {
        let serialized = [7, 0, 0, 0, 25, 1, 2];

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
        let serialized = [7, 0, 0, 0, 25, 1, 3];

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
