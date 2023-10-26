use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SyncReply {
    pub serial: u32,
}

impl MessageOps for SyncReply {
    fn kind(&self) -> MessageKind {
        MessageKind::SyncReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SyncReply);

        serializer.put_varint_u32_le(self.serial);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithoutValueDeserializer::new(buf, MessageKind::SyncReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for SyncReply {}

impl From<SyncReply> for Message {
    fn from(msg: SyncReply) -> Self {
        Self::SyncReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SyncReply;

    #[test]
    fn sync_reply() {
        let serialized = [6, 0, 0, 0, 31, 1];

        let msg = SyncReply { serial: 1 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SyncReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
