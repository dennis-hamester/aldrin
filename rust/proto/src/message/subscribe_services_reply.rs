use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubscribeServicesReply {
    pub serial: u32,
}

impl MessageOps for SubscribeServicesReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeServicesReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeServicesReply);
        serializer.put_varint_u32_le(self.serial);
        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeServicesReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for SubscribeServicesReply {}

impl From<SubscribeServicesReply> for Message {
    fn from(msg: SubscribeServicesReply) -> Self {
        Self::SubscribeServicesReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeServicesReply;

    #[test]
    fn subscribe_services_reply() {
        let serialized = [6, 0, 0, 0, 17, 0];

        let msg = SubscribeServicesReply { serial: 0 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeServicesReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
