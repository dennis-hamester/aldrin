use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::bus_listener::BusListenerScope;
use crate::ids::BusListenerCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct StartBusListener {
    pub serial: u32,
    pub cookie: BusListenerCookie,
    pub scope: BusListenerScope,
}

impl MessageOps for StartBusListener {
    fn kind(&self) -> MessageKind {
        MessageKind::StartBusListener
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::StartBusListener);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.scope);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::StartBusListener)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(BusListenerCookie)?;
        let scope = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;

        Ok(Self {
            serial,
            cookie,
            scope,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for StartBusListener {}

impl From<StartBusListener> for Message {
    fn from(msg: StartBusListener) -> Self {
        Self::StartBusListener(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::StartBusListener;
    use crate::bus_listener::BusListenerScope;
    use crate::ids::BusListenerCookie;
    use uuid::uuid;

    #[test]
    fn current() {
        let serialized = [
            23, 0, 0, 0, 40, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 0,
        ];

        let msg = StartBusListener {
            serial: 0,
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            scope: BusListenerScope::Current,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListener(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn new() {
        let serialized = [
            23, 0, 0, 0, 40, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 1,
        ];

        let msg = StartBusListener {
            serial: 0,
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            scope: BusListenerScope::New,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListener(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn all() {
        let serialized = [
            23, 0, 0, 0, 40, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 2,
        ];

        let msg = StartBusListener {
            serial: 0,
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            scope: BusListenerScope::All,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListener(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
