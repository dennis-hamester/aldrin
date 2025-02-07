use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct AbortFunctionCall {
    pub serial: u32,
}

impl MessageOps for AbortFunctionCall {
    fn kind(&self) -> MessageKind {
        MessageKind::AbortFunctionCall
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::AbortFunctionCall);

        serializer.put_varint_u32_le(self.serial);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::AbortFunctionCall)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for AbortFunctionCall {}

impl From<AbortFunctionCall> for Message {
    fn from(msg: AbortFunctionCall) -> Self {
        Self::AbortFunctionCall(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::AbortFunctionCall;

    #[test]
    fn sync() {
        let serialized = [6, 0, 0, 0, 48, 1];

        let msg = AbortFunctionCall { serial: 1 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AbortFunctionCall(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
