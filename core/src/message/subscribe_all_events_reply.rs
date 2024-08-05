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
pub enum SubscribeAllEventsResult {
    Ok = 0,
    InvalidService = 1,
    NotSupported = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeAllEventsReply {
    pub serial: u32,
    pub result: SubscribeAllEventsResult,
}

impl MessageOps for SubscribeAllEventsReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeAllEventsReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeAllEventsReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeAllEventsReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for SubscribeAllEventsReply {}

impl From<SubscribeAllEventsReply> for Message {
    fn from(msg: SubscribeAllEventsReply) -> Self {
        Self::SubscribeAllEventsReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{SubscribeAllEventsReply, SubscribeAllEventsResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 59, 1, 0];

        let msg = SubscribeAllEventsReply {
            serial: 1,
            result: SubscribeAllEventsResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [7, 0, 0, 0, 59, 1, 1];

        let msg = SubscribeAllEventsReply {
            serial: 1,
            result: SubscribeAllEventsResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn not_supported() {
        let serialized = [7, 0, 0, 0, 59, 1, 2];

        let msg = SubscribeAllEventsReply {
            serial: 1,
            result: SubscribeAllEventsResult::NotSupported,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
