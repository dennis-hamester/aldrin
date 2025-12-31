use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateBusListener {
    pub serial: u32,
}

impl MessageOps for CreateBusListener {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateBusListener
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateBusListener);

        serializer.put_varint_u32_le(self.serial);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateBusListener)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { serial })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for CreateBusListener {}

impl From<CreateBusListener> for Message {
    fn from(msg: CreateBusListener) -> Self {
        Self::CreateBusListener(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::CreateBusListener;

    #[test]
    fn sync() {
        let serialized = [6, 0, 0, 0, 33, 1];

        let msg = CreateBusListener { serial: 1 };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateBusListener(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
