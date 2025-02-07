use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::{ObjectCookie, ServiceUuid};
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::service_info::ServiceInfo;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateService2 {
    pub serial: u32,
    pub object_cookie: ObjectCookie,
    pub uuid: ServiceUuid,
    pub value: SerializedValue,
}

impl CreateService2 {
    pub fn with_serialize_info(
        serial: u32,
        object_cookie: ObjectCookie,
        uuid: ServiceUuid,
        info: ServiceInfo,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(&info)?;

        Ok(Self {
            serial,
            object_cookie,
            uuid,
            value,
        })
    }

    pub fn deserialize_info(&self) -> Result<ServiceInfo, DeserializeError> {
        self.value.deserialize()
    }
}

impl MessageOps for CreateService2 {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateService2
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer =
            MessageSerializer::with_value(self.value, MessageKind::CreateService2)?;

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.object_cookie.0);
        serializer.put_uuid(self.uuid.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::CreateService2)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let object_cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
        let uuid = deserializer.try_get_uuid().map(ServiceUuid)?;
        let value = deserializer.finish()?;

        Ok(Self {
            serial,
            object_cookie,
            uuid,
            value,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }
}

impl Sealed for CreateService2 {}

impl From<CreateService2> for Message {
    fn from(msg: CreateService2) -> Self {
        Self::CreateService2(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::CreateService2;
    use crate::ids::{ObjectCookie, ServiceUuid};
    use crate::service_info::ServiceInfo;
    use uuid::uuid;

    #[test]
    fn create_service2() {
        let serialized = [
            51, 0, 0, 0, 52, 9, 0, 0, 0, 39, 3, 0, 7, 2, 1, 0, 2, 0, 1, 0xb7, 0xc3, 0xbe, 0x13,
            0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b, 0xd3, 0xef,
            0xd0, 0x0b, 0x7a, 0x7b, 0x4b, 0xf7, 0xbd, 0xd3, 0x3c, 0x66, 0x32, 0x47, 0x33, 0x47,
        ];

        let msg = CreateService2::with_serialize_info(
            1,
            ObjectCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
            ServiceUuid(uuid!("d3efd00b-7a7b-4bf7-bdd3-3c6632473347")),
            ServiceInfo::new(2),
        )
        .unwrap();

        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateService2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
