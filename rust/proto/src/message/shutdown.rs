use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Shutdown;

impl MessageOps for Shutdown {
    fn kind(&self) -> MessageKind {
        MessageKind::Shutdown
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        MessageSerializer::without_value(MessageKind::Shutdown).finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        MessageWithoutValueDeserializer::new(buf, MessageKind::Shutdown)?.finish()?;
        Ok(Self)
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for Shutdown {}

impl From<Shutdown> for Message {
    fn from(msg: Shutdown) -> Self {
        Self::Shutdown(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::Shutdown;

    #[test]
    fn shutdown() {
        let serialized = [5, 0, 0, 0, 2];

        let msg = Shutdown;
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::Shutdown(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
