use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct Sync {
    pub serial: u32,
}

impl MessageOps for Sync {
    fn kind(&self) -> MessageKind {
        MessageKind::Sync
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::Sync);

        serializer.put_varint_u32_le(self.serial);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithoutValueDeserializer::new(buf, MessageKind::Sync)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for Sync {}

impl From<Sync> for Message {
    fn from(msg: Sync) -> Self {
        Self::Sync(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::Sync;

    #[test]
    fn sync() {
        let serialized = [6, 0, 0, 0, 41, 1];

        let msg = Sync { serial: 1 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::Sync(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
