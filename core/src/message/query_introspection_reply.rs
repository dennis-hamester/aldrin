use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
#[cfg(feature = "introspection")]
use crate::error::SerializeError;
#[cfg(feature = "introspection")]
use crate::introspection::Introspection;
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum QueryIntrospectionReplyKind {
    Ok = 0,
    Unavailable = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum QueryIntrospectionResult {
    Ok(SerializedValue),
    Unavailable,
}

impl QueryIntrospectionResult {
    #[cfg(feature = "introspection")]
    pub fn ok_with_serialize_introspection(
        introspection: &Introspection,
    ) -> Result<Self, SerializeError> {
        SerializedValue::serialize(introspection).map(Self::Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryIntrospectionReply {
    pub serial: u32,
    pub result: QueryIntrospectionResult,
}

impl QueryIntrospectionReply {
    #[cfg(feature = "introspection")]
    pub fn ok_with_serialize_introspection(
        serial: u32,
        introspection: &Introspection,
    ) -> Result<Self, SerializeError> {
        let result = QueryIntrospectionResult::ok_with_serialize_introspection(introspection)?;
        Ok(Self { serial, result })
    }
}

impl MessageOps for QueryIntrospectionReply {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryIntrospectionReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let serializer = match self.result {
            QueryIntrospectionResult::Ok(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::QueryIntrospectionReply)?;

                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(QueryIntrospectionReplyKind::Ok);

                serializer
            }

            QueryIntrospectionResult::Unavailable => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::QueryIntrospectionReply);

                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(QueryIntrospectionReplyKind::Unavailable);

                serializer
            }
        };

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithValueDeserializer::new(buf, MessageKind::QueryIntrospectionReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        match deserializer.try_get_discriminant_u8()? {
            QueryIntrospectionReplyKind::Ok => {
                let value = deserializer.finish()?;

                Ok(Self {
                    serial,
                    result: QueryIntrospectionResult::Ok(value),
                })
            }

            QueryIntrospectionReplyKind::Unavailable => {
                deserializer.finish_discard_value()?;

                Ok(Self {
                    serial,
                    result: QueryIntrospectionResult::Unavailable,
                })
            }
        }
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        match self.result {
            QueryIntrospectionResult::Ok(ref value) => Some(value),
            QueryIntrospectionResult::Unavailable => None,
        }
    }
}

impl Sealed for QueryIntrospectionReply {}

impl From<QueryIntrospectionReply> for Message {
    fn from(msg: QueryIntrospectionReply) -> Self {
        Self::QueryIntrospectionReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{
        assert_deserialize_eq, assert_deserialize_eq_with_value, assert_serialize_eq,
    };
    use super::super::Message;
    use super::{QueryIntrospectionReply, QueryIntrospectionResult};
    use crate::serialized_value::SerializedValue;

    #[test]
    fn ok() {
        let serialized = [13, 0, 0, 0, 51, 2, 0, 0, 0, 3, 4, 1, 0];
        let value = 4u8;

        let msg = QueryIntrospectionReply {
            serial: 1,
            result: QueryIntrospectionResult::Ok(SerializedValue::serialize(&value).unwrap()),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::QueryIntrospectionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn unavailable() {
        let serialized = [12, 0, 0, 0, 51, 1, 0, 0, 0, 0, 1, 1];

        let msg = QueryIntrospectionReply {
            serial: 1,
            result: QueryIntrospectionResult::Unavailable,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryIntrospectionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
