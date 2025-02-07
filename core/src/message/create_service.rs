use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{ObjectCookie, SerializedValueSlice, ServiceUuid};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: ObjectCookie,
    pub uuid: ServiceUuid,
    pub version: u32,
}

impl MessageOps for CreateService {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateService
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateService);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.object_cookie.0);
        serializer.put_uuid(self.uuid.0);
        serializer.put_varint_u32_le(self.version);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateService)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let object_cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
        let uuid = deserializer.try_get_uuid().map(ServiceUuid)?;
        let version = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            object_cookie,
            uuid,
            version,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for CreateService {}

impl From<CreateService> for Message {
    fn from(msg: CreateService) -> Self {
        Self::CreateService(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::CreateService;
    use crate::{ObjectCookie, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn create_service() {
        let serialized = [
            39, 0, 0, 0, 7, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b, 0xd3, 0xef, 0xd0, 0x0b, 0x7a, 0x7b, 0x4b, 0xf7, 0xbd,
            0xd3, 0x3c, 0x66, 0x32, 0x47, 0x33, 0x47, 2,
        ];

        let msg = CreateService {
            serial: 1,
            object_cookie: ObjectCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
            uuid: ServiceUuid(uuid!("d3efd00b-7a7b-4bf7-bdd3-3c6632473347")),
            version: 2,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateService(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
