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
pub enum SubscribeServiceResult {
    Ok = 0,
    InvalidService = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeServiceReply {
    pub serial: u32,
    pub result: SubscribeServiceResult,
}

impl MessageOps for SubscribeServiceReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeServiceReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeServiceReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeServiceReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for SubscribeServiceReply {}

impl From<SubscribeServiceReply> for Message {
    fn from(msg: SubscribeServiceReply) -> Self {
        Self::SubscribeServiceReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{SubscribeServiceReply, SubscribeServiceResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 56, 1, 0];

        let msg = SubscribeServiceReply {
            serial: 1,
            result: SubscribeServiceResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [7, 0, 0, 0, 56, 1, 1];

        let msg = SubscribeServiceReply {
            serial: 1,
            result: SubscribeServiceResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
