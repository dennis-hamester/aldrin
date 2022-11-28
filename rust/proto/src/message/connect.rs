use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::value_serializer::Serialize;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect {
    pub version: u32,
    pub value: BytesMut,
}

impl Connect {
    pub fn with_serialize_value<T: Serialize + ?Sized>(
        version: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = super::message_buf_with_serialize_value(value)?;
        Ok(Self { version, value })
    }

    fn value_buf(&self) -> &[u8] {
        MessageWithValueDeserializer::value_buf(&self.value)
    }
}

impl MessageOps for Connect {
    fn kind(&self) -> MessageKind {
        MessageKind::Connect
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::Connect)?;

        serializer.put_varint_u32_le(self.version);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::Connect)?;

        let version = deserializer.try_get_varint_u32_le()?;
        let value = deserializer.finish()?;

        Ok(Self { version, value })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        Some(self.value_buf())
    }
}

impl Sealed for Connect {}

impl From<Connect> for Message {
    fn from(msg: Connect) -> Self {
        Self::Connect(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::Connect;

    #[test]
    fn connect() {
        let serialized = [12, 0, 0, 0, 0, 2, 0, 0, 0, 3, 4, 1];
        let value = 4u8;

        let msg = Connect::with_serialize_value(1, &value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::Connect(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
