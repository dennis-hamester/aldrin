use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ServiceCookie;
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnsubscribeEvent {
    pub service_cookie: ServiceCookie,
    pub event: u32,
}

impl MessageOps for UnsubscribeEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::UnsubscribeEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::UnsubscribeEvent);

        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.event);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::UnsubscribeEvent)?;

        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
        let event = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self {
            service_cookie,
            event,
        })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for UnsubscribeEvent {}

impl From<UnsubscribeEvent> for Message {
    fn from(msg: UnsubscribeEvent) -> Self {
        Self::UnsubscribeEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::UnsubscribeEvent;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn unsubscribe_event() {
        let serialized = [
            22, 0, 0, 0, 25, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc, 0x4e,
            0x0e, 0x84, 0x80, 0xdf, 0xad, 1,
        ];

        let msg = UnsubscribeEvent {
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
            event: 1,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::UnsubscribeEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
