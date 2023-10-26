use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum CloseChannelEndResult {
    Ok = 0,
    InvalidChannel = 1,
    ForeignChannel = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CloseChannelEndReply {
    pub serial: u32,
    pub result: CloseChannelEndResult,
}

impl MessageOps for CloseChannelEndReply {
    fn kind(&self) -> MessageKind {
        MessageKind::CloseChannelEndReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CloseChannelEndReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CloseChannelEndReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for CloseChannelEndReply {}

impl From<CloseChannelEndReply> for Message {
    fn from(msg: CloseChannelEndReply) -> Self {
        Self::CloseChannelEndReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{CloseChannelEndReply, CloseChannelEndResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 22, 1, 0];

        let msg = CloseChannelEndReply {
            serial: 1,
            result: CloseChannelEndResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CloseChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_channel() {
        let serialized = [7, 0, 0, 0, 22, 1, 1];

        let msg = CloseChannelEndReply {
            serial: 1,
            result: CloseChannelEndResult::InvalidChannel,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CloseChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn foreign_channel() {
        let serialized = [7, 0, 0, 0, 22, 1, 2];

        let msg = CloseChannelEndReply {
            serial: 1,
            result: CloseChannelEndResult::ForeignChannel,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CloseChannelEndReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
