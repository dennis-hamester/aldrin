use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum UnsubscribeAllEventsResult {
    Ok = 0,
    InvalidService = 1,
    NotSupported = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct UnsubscribeAllEventsReply {
    pub serial: u32,
    pub result: UnsubscribeAllEventsResult,
}

impl MessageOps for UnsubscribeAllEventsReply {
    fn kind(&self) -> MessageKind {
        MessageKind::UnsubscribeAllEventsReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer =
            MessageSerializer::without_value(MessageKind::UnsubscribeAllEventsReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::UnsubscribeAllEventsReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

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

impl Sealed for UnsubscribeAllEventsReply {}

impl From<UnsubscribeAllEventsReply> for Message {
    fn from(msg: UnsubscribeAllEventsReply) -> Self {
        Self::UnsubscribeAllEventsReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::{UnsubscribeAllEventsReply, UnsubscribeAllEventsResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 61, 1, 0];

        let msg = UnsubscribeAllEventsReply {
            serial: 1,
            result: UnsubscribeAllEventsResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [7, 0, 0, 0, 61, 1, 1];

        let msg = UnsubscribeAllEventsReply {
            serial: 1,
            result: UnsubscribeAllEventsResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn not_supported() {
        let serialized = [7, 0, 0, 0, 61, 1, 2];

        let msg = UnsubscribeAllEventsReply {
            serial: 1,
            result: UnsubscribeAllEventsResult::NotSupported,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeAllEventsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
