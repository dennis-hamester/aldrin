use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageWithoutValueDeserializer,
};
use crate::ids::BusListenerCookie;
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ClearBusListenerFilters {
    pub cookie: BusListenerCookie,
}

impl MessageOps for ClearBusListenerFilters {
    fn kind(&self) -> MessageKind {
        MessageKind::ClearBusListenerFilters
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ClearBusListenerFilters);

        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ClearBusListenerFilters)?;

        let cookie = deserializer.try_get_uuid().map(BusListenerCookie)?;

        deserializer.finish()?;
        Ok(Self { cookie })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for ClearBusListenerFilters {}

impl From<ClearBusListenerFilters> for Message {
    fn from(msg: ClearBusListenerFilters) -> Self {
        Self::ClearBusListenerFilters(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ClearBusListenerFilters;
    use crate::ids::BusListenerCookie;
    use uuid::uuid;

    #[test]
    fn clear_bus_listener_filters() {
        let serialized = [
            21, 0, 0, 0, 39, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72,
        ];

        let msg = ClearBusListenerFilters {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClearBusListenerFilters(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
