use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer, OptionKind,
};
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeObjects {
    pub serial: Option<u32>,
}

impl MessageOps for SubscribeObjects {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeObjects
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeObjects);

        match self.serial {
            None => {
                serializer.put_discriminant_u8(OptionKind::None);
            }

            Some(serial) => {
                serializer.put_discriminant_u8(OptionKind::Some);
                serializer.put_varint_u32_le(serial);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeObjects)?;

        let serial = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for SubscribeObjects {}

impl From<SubscribeObjects> for Message {
    fn from(msg: SubscribeObjects) -> Self {
        Self::SubscribeObjects(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeObjects;

    #[test]
    fn no_serial() {
        let serialized = [6, 0, 0, 0, 7, 0];

        let msg = SubscribeObjects { serial: None };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeObjects(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn serial() {
        let serialized = [7, 0, 0, 0, 7, 1, 2];

        let msg = SubscribeObjects { serial: Some(2) };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeObjects(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}