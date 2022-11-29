use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceCookie;
use crate::value_serializer::Serialize;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitEvent {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub value: BytesMut,
}

impl EmitEvent {
    pub fn with_serialize_value<T: Serialize + ?Sized>(
        service_cookie: ServiceCookie,
        event: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = super::message_buf_with_serialize_value(value)?;
        Ok(Self {
            service_cookie,
            event,
            value,
        })
    }

    fn value(&self) -> &[u8] {
        MessageWithValueDeserializer::value_buf(&self.value)
    }
}

impl MessageOps for EmitEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::EmitEvent
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::EmitEvent)?;

        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.event);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::EmitEvent)?;

        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
        let event = deserializer.try_get_varint_u32_le()?;
        let value = deserializer.finish()?;

        Ok(Self {
            service_cookie,
            event,
            value,
        })
    }

    fn value_opt(&self) -> Option<&[u8]> {
        Some(self.value())
    }
}

impl Sealed for EmitEvent {}

impl From<EmitEvent> for Message {
    fn from(msg: EmitEvent) -> Self {
        Self::EmitEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::EmitEvent;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn emit_event() {
        let serialized = [
            28, 0, 0, 0, 26, 2, 0, 0, 0, 3, 4, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 1,
        ];
        let value = 4u8;

        let msg = EmitEvent::with_serialize_value(
            ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            1,
            &value,
        )
        .unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::EmitEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
