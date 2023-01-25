use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeObjectsReply {
    pub serial: u32,
}

impl MessageOps for SubscribeObjectsReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeObjectsReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeObjectsReply);
        serializer.put_varint_u32_le(self.serial);
        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeObjectsReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for SubscribeObjectsReply {}

impl From<SubscribeObjectsReply> for Message {
    fn from(msg: SubscribeObjectsReply) -> Self {
        Self::SubscribeObjectsReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeObjectsReply;

    #[test]
    fn subscribe_objects_reply() {
        let serialized = [6, 0, 0, 0, 8, 0];

        let msg = SubscribeObjectsReply { serial: 0 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeObjectsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
