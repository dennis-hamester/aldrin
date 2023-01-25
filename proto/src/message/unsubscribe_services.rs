use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct UnsubscribeServices;

impl MessageOps for UnsubscribeServices {
    fn kind(&self) -> MessageKind {
        MessageKind::UnsubscribeServices
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        MessageSerializer::without_value(MessageKind::UnsubscribeServices).finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        MessageWithoutValueDeserializer::new(buf, MessageKind::UnsubscribeServices)?.finish()?;
        Ok(Self)
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for UnsubscribeServices {}

impl From<UnsubscribeServices> for Message {
    fn from(msg: UnsubscribeServices) -> Self {
        Self::UnsubscribeServices(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::UnsubscribeServices;

    #[test]
    fn unsubscribe_services() {
        let serialized = [5, 0, 0, 0, 18];

        let msg = UnsubscribeServices;
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeServices(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
