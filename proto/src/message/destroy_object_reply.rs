use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::value::SerializedValue;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum DestroyObjectResult {
    Ok = 0,
    InvalidObject = 1,
    ForeignObject = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

impl MessageOps for DestroyObjectReply {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyObjectReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyObjectReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyObjectReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for DestroyObjectReply {}

impl From<DestroyObjectReply> for Message {
    fn from(msg: DestroyObjectReply) -> Self {
        Self::DestroyObjectReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{DestroyObjectReply, DestroyObjectResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 6, 1, 0];

        let msg = DestroyObjectReply {
            serial: 1,
            result: DestroyObjectResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_object() {
        let serialized = [7, 0, 0, 0, 6, 1, 1];

        let msg = DestroyObjectReply {
            serial: 1,
            result: DestroyObjectResult::InvalidObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn foreign_object() {
        let serialized = [7, 0, 0, 0, 6, 1, 2];

        let msg = DestroyObjectReply {
            serial: 1,
            result: DestroyObjectResult::ForeignObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
