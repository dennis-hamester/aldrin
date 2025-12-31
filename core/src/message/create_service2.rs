use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::{ObjectCookie, SerializedValue, SerializedValueSlice, ServiceUuid};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateService2 {
    pub serial: u32,
    pub object_cookie: ObjectCookie,
    pub uuid: ServiceUuid,
    pub value: SerializedValue,
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

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
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
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::CreateService2;
    use crate::{ObjectCookie, SerializedValue, ServiceInfo, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn create_service2() {
        let serialized = [
            48, 0, 0, 0, 52, 6, 0, 0, 0, 65, 1, 0, 7, 2, 0, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77,
            0x46, 0x6e, 0xb4, 0xbf, 0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b, 0xd3, 0xef, 0xd0, 0x0b,
            0x7a, 0x7b, 0x4b, 0xf7, 0xbd, 0xd3, 0x3c, 0x66, 0x32, 0x47, 0x33, 0x47,
        ];
        let value = ServiceInfo::new(2);

        let msg = CreateService2 {
            serial: 1,
            object_cookie: ObjectCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
            uuid: ServiceUuid(uuid!("d3efd00b-7a7b-4bf7-bdd3-3c6632473347")),
            value: SerializedValue::serialize(value).unwrap(),
        };

        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CreateService2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
