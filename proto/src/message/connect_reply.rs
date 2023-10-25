use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::error::SerializeError;
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_serializer::Serialize;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum ConnectReplyKind {
    Ok = 0,
    VersionMismatch = 1,
    Rejected = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum ConnectReply {
    Ok(SerializedValue),
    VersionMismatch(u32),
    Rejected(SerializedValue),
}

impl ConnectReply {
    pub fn ok_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(value)?;
        Ok(Self::Ok(value))
    }

    pub fn rejected_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(value)?;
        Ok(Self::Rejected(value))
    }
}

impl MessageOps for ConnectReply {
    fn kind(&self) -> MessageKind {
        MessageKind::ConnectReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        match self {
            Self::Ok(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::ConnectReply)?;
                serializer.put_discriminant_u8(ConnectReplyKind::Ok);
                serializer.finish()
            }

            Self::VersionMismatch(version) => {
                let mut serializer = MessageSerializer::with_none_value(MessageKind::ConnectReply);
                serializer.put_discriminant_u8(ConnectReplyKind::VersionMismatch);
                serializer.put_varint_u32_le(version);
                serializer.finish()
            }

            Self::Rejected(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::ConnectReply)?;
                serializer.put_discriminant_u8(ConnectReplyKind::Rejected);
                serializer.finish()
            }
        }
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
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

    fn value(&self) -> Option<&SerializedValueSlice> {
        match self {
            Self::Ok(value) | Self::Rejected(value) => Some(value),
            Self::VersionMismatch(_) => None,
        }
    }
}

impl Sealed for ConnectReply {}

impl From<ConnectReply> for Message {
    fn from(msg: ConnectReply) -> Self {
        Self::ConnectReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{
        assert_deserialize_eq, assert_deserialize_eq_with_value, assert_serialize_eq,
    };
    use super::super::Message;
    use super::ConnectReply;

    #[test]
    fn ok() {
        let serialized = [12, 0, 0, 0, 1, 2, 0, 0, 0, 3, 4, 0];
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
        let serialized = [12, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 2];

        let msg = ConnectReply::VersionMismatch(2);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ConnectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn rejected() {
        let serialized = [12, 0, 0, 0, 1, 2, 0, 0, 0, 3, 4, 2];
        let value = 4u8;

        let msg = ConnectReply::rejected_with_serialize_value(&value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
