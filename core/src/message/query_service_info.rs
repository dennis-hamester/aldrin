use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::ids::ServiceCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryServiceInfo {
    pub serial: u32,
    pub cookie: ServiceCookie,
}

impl MessageOps for QueryServiceInfo {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryServiceInfo
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::QueryServiceInfo);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::QueryServiceInfo)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        deserializer.finish()?;
        Ok(Self { serial, cookie })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for QueryServiceInfo {}

impl From<QueryServiceInfo> for Message {
    fn from(msg: QueryServiceInfo) -> Self {
        Self::QueryServiceInfo(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::QueryServiceInfo;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn query_service_info() {
        let serialized = [
            22, 0, 0, 0, 53, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = QueryServiceInfo {
            serial: 1,
            cookie: ServiceCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryServiceInfo(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
