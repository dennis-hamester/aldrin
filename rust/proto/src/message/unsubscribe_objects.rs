use super::message_ops::Sealed;
use super::{MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnsubscribeObjects;

impl MessageOps for UnsubscribeObjects {
    fn kind(&self) -> MessageKind {
        MessageKind::UnsubscribeObjects
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        Ok(MessageSerializer::without_value(MessageKind::UnsubscribeObjects).finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        MessageWithoutValueDeserializer::new(buf, MessageKind::UnsubscribeObjects)?.finish()?;
        Ok(Self)
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for UnsubscribeObjects {}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::UnsubscribeObjects;

    #[test]
    fn unsubscribe_objects() {
        let serialized = [9];

        let msg = UnsubscribeObjects;
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeObjects(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
