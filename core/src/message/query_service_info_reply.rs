use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::SerializeError;
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::service_info::ServiceInfo;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum QueryServiceInfoReplyKind {
    Ok = 0,
    InvalidService = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum QueryServiceInfoResult {
    Ok(SerializedValue),
    InvalidService,
}

impl QueryServiceInfoResult {
    pub fn ok_with_serialize_info(info: ServiceInfo) -> Result<Self, SerializeError> {
        SerializedValue::serialize(&info).map(Self::Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryServiceInfoReply {
    pub serial: u32,
    pub result: QueryServiceInfoResult,
}

impl QueryServiceInfoReply {
    pub fn ok_with_serialize_info(serial: u32, info: ServiceInfo) -> Result<Self, SerializeError> {
        let result = QueryServiceInfoResult::ok_with_serialize_info(info)?;
        Ok(Self { serial, result })
    }
}

impl MessageOps for QueryServiceInfoReply {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryServiceInfoReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let serializer = match self.result {
            QueryServiceInfoResult::Ok(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::QueryServiceInfoReply)?;

                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(QueryServiceInfoReplyKind::Ok);

                serializer
            }

            QueryServiceInfoResult::InvalidService => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::QueryServiceInfoReply);

                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(QueryServiceInfoReplyKind::InvalidService);

                serializer
            }
        };

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithValueDeserializer::new(buf, MessageKind::QueryServiceInfoReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        match deserializer.try_get_discriminant_u8()? {
            QueryServiceInfoReplyKind::Ok => {
                let value = deserializer.finish()?;

                Ok(Self {
                    serial,
                    result: QueryServiceInfoResult::Ok(value),
                })
            }

            QueryServiceInfoReplyKind::InvalidService => {
                deserializer.finish_discard_value()?;

                Ok(Self {
                    serial,
                    result: QueryServiceInfoResult::InvalidService,
                })
            }
        }
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        match self.result {
            QueryServiceInfoResult::Ok(ref value) => Some(value),
            QueryServiceInfoResult::InvalidService => None,
        }
    }
}

impl Sealed for QueryServiceInfoReply {}

impl From<QueryServiceInfoReply> for Message {
    fn from(msg: QueryServiceInfoReply) -> Self {
        Self::QueryServiceInfoReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{
        assert_deserialize_eq, assert_deserialize_eq_with_value, assert_serialize_eq,
    };
    use super::super::Message;
    use super::{QueryServiceInfoReply, QueryServiceInfoResult};
    use crate::ids::TypeId;
    use crate::service_info::ServiceInfo;
    use uuid::uuid;

    #[test]
    fn ok() {
        let serialized = [
            35, 0, 0, 0, 54, 24, 0, 0, 0, 39, 2, 0, 7, 2, 1, 1, 14, 0xcf, 0x41, 0xc6, 0x88, 0x49,
            0x76, 0x46, 0xa5, 0x8e, 0x2d, 0x48, 0x71, 0x02, 0x58, 0xbc, 0x2c, 1, 0,
        ];
        let info =
            ServiceInfo::with_type_id(2, TypeId(uuid!("cf41c688-4976-46a5-8e2d-48710258bc2c")));

        let msg = QueryServiceInfoReply::ok_with_serialize_info(1, info).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &info);

        let msg = Message::QueryServiceInfoReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &info);
    }

    #[test]
    fn invalid_service() {
        let serialized = [12, 0, 0, 0, 54, 1, 0, 0, 0, 0, 1, 1];

        let msg = QueryServiceInfoReply {
            serial: 1,
            result: QueryServiceInfoResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryServiceInfoReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
