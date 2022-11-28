use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ObjectUuid;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueryObject {
    pub serial: u32,
    pub uuid: ObjectUuid,
    pub with_services: bool,
}

impl MessageOps for QueryObject {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryObject
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::QueryObject);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.uuid.0);
        serializer.put_bool(self.with_services);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer = MessageWithoutValueDeserializer::new(buf, MessageKind::QueryObject)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let uuid = deserializer.try_get_uuid().map(ObjectUuid)?;
        let with_services = deserializer.try_get_bool()?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            uuid,
            with_services,
        })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for QueryObject {}

impl From<QueryObject> for Message {
    fn from(msg: QueryObject) -> Self {
        Self::QueryObject(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::QueryObject;
    use crate::ids::ObjectUuid;
    use uuid::uuid;

    #[test]
    fn query_object() {
        let serialized = [
            23, 0, 0, 0, 27, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b, 0,
        ];

        let msg = QueryObject {
            serial: 1,
            uuid: ObjectUuid(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
            with_services: false,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryObject(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
