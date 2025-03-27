use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{BusListenerCookie, SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct BusListenerCurrentFinished {
    pub cookie: BusListenerCookie,
}

impl MessageOps for BusListenerCurrentFinished {
    fn kind(&self) -> MessageKind {
        MessageKind::BusListenerCurrentFinished
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer =
            MessageSerializer::without_value(MessageKind::BusListenerCurrentFinished);

        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::BusListenerCurrentFinished)?;

        let cookie = deserializer.try_get_uuid().map(BusListenerCookie)?;

        deserializer.finish()?;
        Ok(Self { cookie })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for BusListenerCurrentFinished {}

impl From<BusListenerCurrentFinished> for Message {
    fn from(msg: BusListenerCurrentFinished) -> Self {
        Self::BusListenerCurrentFinished(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::BusListenerCurrentFinished;
    use crate::BusListenerCookie;
    use uuid::uuid;

    #[test]
    fn bus_listener_current_finished() {
        let serialized = [
            21, 0, 0, 0, 45, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72,
        ];

        let msg = BusListenerCurrentFinished {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::BusListenerCurrentFinished(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
