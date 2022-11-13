use super::message_ops::Sealed;
use super::{MessageKind, MessageOps, MessageSerializer, MessageWithValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::value_serializer::Serialize;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ConnectReplyKind {
    Ok = 0,
    VersionMismatch = 1,
    Rejected = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectReply {
    Ok(BytesMut),
    VersionMismatch(u32),
    Rejected(BytesMut),
}

impl ConnectReply {
    pub fn ok_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = super::message_buf_with_serialize_value(value)?;
        Ok(Self::Ok(value))
    }

    pub fn rejected_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = super::message_buf_with_serialize_value(value)?;
        Ok(Self::Rejected(value))
    }

    fn value_buf(&self) -> &[u8] {
        match self {
            Self::Ok(value) | Self::Rejected(value) => {
                debug_assert!(value.len() >= 6);
                &value[5..]
            }

            Self::VersionMismatch(_) => &[0],
        }
    }
}

impl MessageOps for ConnectReply {
    fn kind(&self) -> MessageKind {
        MessageKind::ConnectReply
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        match self {
            Self::Ok(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::ConnectReply)?;
                serializer.put_discriminant_u8(ConnectReplyKind::Ok);
                Ok(serializer.finish())
            }

            Self::VersionMismatch(version) => {
                let mut serializer = MessageSerializer::with_empty_value(MessageKind::ConnectReply);
                serializer.put_discriminant_u8(ConnectReplyKind::VersionMismatch);
                serializer.put_varint_u32_le(version);
                Ok(serializer.finish())
            }

            Self::Rejected(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::ConnectReply)?;
                serializer.put_discriminant_u8(ConnectReplyKind::Rejected);
                Ok(serializer.finish())
            }
        }
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::ConnectReply)?;

        match deserializer.try_get_discriminant_u8()? {
            ConnectReplyKind::Ok => deserializer.finish().map(Self::Ok),

            ConnectReplyKind::VersionMismatch => {
                let version = deserializer.try_get_varint_u32_le()?;
                deserializer.finish_discard_value()?;
                Ok(Self::VersionMismatch(version))
            }

            ConnectReplyKind::Rejected => deserializer.finish().map(Self::Rejected),
        }
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        Some(self.value_buf())
    }
}

impl Sealed for ConnectReply {}

#[cfg(test)]
mod test {
    use super::super::test::{
        assert_deserialize_eq, assert_deserialize_eq_with_value, assert_serialize_eq,
    };
    use super::super::Message;
    use super::ConnectReply;

    #[test]
    fn ok() {
        let serialized = [1, 2, 0, 0, 0, 3, 4, 0];
        let value = 4u8;

        let msg = ConnectReply::ok_with_serialize_value(&value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn version_mismatch() {
        let serialized = [1, 1, 0, 0, 0, 0, 1, 2];

        let msg = ConnectReply::VersionMismatch(2);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ConnectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn rejected() {
        let serialized = [1, 2, 0, 0, 0, 3, 4, 2];
        let value = 4u8;

        let msg = ConnectReply::rejected_with_serialize_value(&value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
