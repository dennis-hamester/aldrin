use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::{ObjectCookie, ServiceCookie, ServiceUuid};
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum QueryObjectReplyKind {
    Cookie = 0,
    Service = 1,
    Done = 2,
    InvalidObject = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum QueryObjectResult {
    Cookie(ObjectCookie),
    Service {
        uuid: ServiceUuid,
        cookie: ServiceCookie,
    },
    Done,
    InvalidObject,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryObjectReply {
    pub serial: u32,
    pub result: QueryObjectResult,
}

impl MessageOps for QueryObjectReply {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryObjectReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::QueryObjectReply);

        serializer.put_varint_u32_le(self.serial);

        match self.result {
            QueryObjectResult::Cookie(cookie) => {
                serializer.put_discriminant_u8(QueryObjectReplyKind::Cookie);
                serializer.put_uuid(cookie.0);
            }

            QueryObjectResult::Service { uuid, cookie } => {
                serializer.put_discriminant_u8(QueryObjectReplyKind::Service);
                serializer.put_uuid(uuid.0);
                serializer.put_uuid(cookie.0);
            }

            QueryObjectResult::Done => {
                serializer.put_discriminant_u8(QueryObjectReplyKind::Done);
            }

            QueryObjectResult::InvalidObject => {
                serializer.put_discriminant_u8(QueryObjectReplyKind::InvalidObject);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::QueryObjectReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let result = match deserializer.try_get_discriminant_u8()? {
            QueryObjectReplyKind::Cookie => {
                let cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
                QueryObjectResult::Cookie(cookie)
            }

            QueryObjectReplyKind::Service => {
                let uuid = deserializer.try_get_uuid().map(ServiceUuid)?;
                let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
                QueryObjectResult::Service { uuid, cookie }
            }

            QueryObjectReplyKind::Done => QueryObjectResult::Done,
            QueryObjectReplyKind::InvalidObject => QueryObjectResult::InvalidObject,
        };

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for QueryObjectReply {}

impl From<QueryObjectReply> for Message {
    fn from(msg: QueryObjectReply) -> Self {
        Self::QueryObjectReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{QueryObjectReply, QueryObjectResult};
    use crate::ids::{ObjectCookie, ServiceCookie, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn cookie() {
        let serialized = [
            23, 0, 0, 0, 28, 1, 0, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf,
            0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = QueryObjectReply {
            serial: 1,
            result: QueryObjectResult::Cookie(ObjectCookie(uuid!(
                "b7c3be13-5377-466e-b4bf-373876523d1b"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn service() {
        let serialized = [
            39, 0, 0, 0, 28, 2, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf,
            0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b, 0x52, 0x8c, 0x0b, 0x73, 0x0b, 0xba, 0x49, 0xa7,
            0xbf, 0xc7, 0x0f, 0xd4, 0xbc, 0x94, 0x42, 0x71,
        ];

        let msg = QueryObjectReply {
            serial: 2,
            result: QueryObjectResult::Service {
                uuid: ServiceUuid(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
                cookie: ServiceCookie(uuid!("528c0b73-0bba-49a7-bfc7-0fd4bc944271")),
            },
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn done() {
        let serialized = [7, 0, 0, 0, 28, 3, 2];

        let msg = QueryObjectReply {
            serial: 3,
            result: QueryObjectResult::Done,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_object() {
        let serialized = [7, 0, 0, 0, 28, 4, 3];

        let msg = QueryObjectReply {
            serial: 4,
            result: QueryObjectResult::InvalidObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
