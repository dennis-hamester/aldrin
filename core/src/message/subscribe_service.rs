use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::ids::ServiceCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeService {
    pub serial: u32,
    pub service_cookie: ServiceCookie,
}

impl MessageOps for SubscribeService {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeService
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeService);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.service_cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeService)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            service_cookie,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for SubscribeService {}

impl From<SubscribeService> for Message {
    fn from(msg: SubscribeService) -> Self {
        Self::SubscribeService(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeService;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn subscribe_service() {
        let serialized = [
            22, 0, 0, 0, 55, 1, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc, 0x4e,
            0x0e, 0x84, 0x80, 0xdf, 0xad,
        ];

        let msg = SubscribeService {
            serial: 1,
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeService(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
