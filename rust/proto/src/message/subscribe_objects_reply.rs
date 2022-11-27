use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubscribeObjectsReply {
    pub serial: u32,
}

impl MessageOps for SubscribeObjectsReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeObjectsReply
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeObjectsReply);
        serializer.put_varint_u32_le(self.serial);
        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeObjectsReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
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
        let serialized = [8, 0];

        let msg = SubscribeObjectsReply { serial: 0 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeObjectsReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
