use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{BusListenerCookie, SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct StopBusListener {
    pub serial: u32,
    pub cookie: BusListenerCookie,
}

impl MessageOps for StopBusListener {
    fn kind(&self) -> MessageKind {
        MessageKind::StopBusListener
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::StopBusListener);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::StopBusListener)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(BusListenerCookie)?;

        deserializer.finish()?;

        Ok(Self { serial, cookie })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for StopBusListener {}

impl From<StopBusListener> for Message {
    fn from(msg: StopBusListener) -> Self {
        Self::StopBusListener(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::StopBusListener;
    use crate::BusListenerCookie;
    use uuid::uuid;

    #[test]
    fn stop_bus_listener() {
        let serialized = [
            22, 0, 0, 0, 42, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72,
        ];

        let msg = StopBusListener {
            serial: 0,
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StopBusListener(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
