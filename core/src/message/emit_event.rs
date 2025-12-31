use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice, ServiceCookie};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct EmitEvent {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub value: SerializedValue,
}

impl MessageOps for EmitEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::EmitEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::EmitEvent)?;

        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.event);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
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

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
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
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::EmitEvent;
    use crate::{SerializedValue, ServiceCookie, tags};
    use uuid::uuid;

    #[test]
    fn emit_event() {
        let serialized = [
            28, 0, 0, 0, 16, 2, 0, 0, 0, 3, 4, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 1,
        ];
        let value = 4u8;

        let msg = EmitEvent {
            service_cookie: ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            event: 1,
            value: SerializedValue::serialize(value).unwrap(),
        };

        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::U8, _>(&msg, serialized, &value);

        let msg = Message::EmitEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::U8, _>(&msg, serialized, &value);
    }
}
