use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnsubscribeObjects;

impl MessageOps for UnsubscribeObjects {
    fn kind(&self) -> MessageKind {
        MessageKind::UnsubscribeObjects
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        MessageSerializer::without_value(MessageKind::UnsubscribeObjects).finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        MessageWithoutValueDeserializer::new(buf, MessageKind::UnsubscribeObjects)?.finish()?;
        Ok(Self)
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for UnsubscribeObjects {}

impl From<UnsubscribeObjects> for Message {
    fn from(msg: UnsubscribeObjects) -> Self {
        Self::UnsubscribeObjects(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::UnsubscribeObjects;

    #[test]
    fn unsubscribe_objects() {
        let serialized = [5, 0, 0, 0, 9];

        let msg = UnsubscribeObjects;
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeObjects(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
