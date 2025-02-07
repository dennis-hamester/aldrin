use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::SerializedValueSlice;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum DestroyServiceResult {
    Ok = 0,
    InvalidService = 1,
    ForeignObject = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

impl MessageOps for DestroyServiceReply {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyServiceReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyServiceReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyServiceReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for DestroyServiceReply {}

impl From<DestroyServiceReply> for Message {
    fn from(msg: DestroyServiceReply) -> Self {
        Self::DestroyServiceReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{DestroyServiceReply, DestroyServiceResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 10, 1, 0];

        let msg = DestroyServiceReply {
            serial: 1,
            result: DestroyServiceResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [7, 0, 0, 0, 10, 1, 1];

        let msg = DestroyServiceReply {
            serial: 1,
            result: DestroyServiceResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn foreign_object() {
        let serialized = [7, 0, 0, 0, 10, 1, 2];

        let msg = DestroyServiceReply {
            serial: 1,
            result: DestroyServiceResult::ForeignObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
