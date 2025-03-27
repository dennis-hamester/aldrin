use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum QueryServiceVersionReplyKind {
    Ok = 0,
    InvalidService = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum QueryServiceVersionResult {
    Ok(u32),
    InvalidService,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryServiceVersionReply {
    pub serial: u32,
    pub result: QueryServiceVersionResult,
}

impl MessageOps for QueryServiceVersionReply {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryServiceVersionReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer =
            MessageSerializer::without_value(MessageKind::QueryServiceVersionReply);

        serializer.put_varint_u32_le(self.serial);

        match self.result {
            QueryServiceVersionResult::Ok(version) => {
                serializer.put_discriminant_u8(QueryServiceVersionReplyKind::Ok);
                serializer.put_varint_u32_le(version);
            }
            QueryServiceVersionResult::InvalidService => {
                serializer.put_discriminant_u8(QueryServiceVersionReplyKind::InvalidService);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::QueryServiceVersionReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let result = match deserializer.try_get_discriminant_u8()? {
            QueryServiceVersionReplyKind::Ok => {
                let version = deserializer.try_get_varint_u32_le()?;
                QueryServiceVersionResult::Ok(version)
            }

            QueryServiceVersionReplyKind::InvalidService => {
                QueryServiceVersionResult::InvalidService
            }
        };

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for QueryServiceVersionReply {}

impl From<QueryServiceVersionReply> for Message {
    fn from(msg: QueryServiceVersionReply) -> Self {
        Self::QueryServiceVersionReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{QueryServiceVersionReply, QueryServiceVersionResult};

    #[test]
    fn ok() {
        let serialized = [8, 0, 0, 0, 18, 1, 0, 2];

        let msg = QueryServiceVersionReply {
            serial: 1,
            result: QueryServiceVersionResult::Ok(2),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryServiceVersionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [7, 0, 0, 0, 18, 1, 1];

        let msg = QueryServiceVersionReply {
            serial: 1,
            result: QueryServiceVersionResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryServiceVersionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
