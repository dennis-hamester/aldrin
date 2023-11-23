use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum StartBusListenerResult {
    Ok = 0,
    InvalidBusListener = 1,
    AlreadyStarted = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct StartBusListenerReply {
    pub serial: u32,
    pub result: StartBusListenerResult,
}

impl MessageOps for StartBusListenerReply {
    fn kind(&self) -> MessageKind {
        MessageKind::StartBusListenerReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::StartBusListenerReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::StartBusListenerReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for StartBusListenerReply {}

impl From<StartBusListenerReply> for Message {
    fn from(msg: StartBusListenerReply) -> Self {
        Self::StartBusListenerReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{StartBusListenerReply, StartBusListenerResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 41, 1, 0];

        let msg = StartBusListenerReply {
            serial: 1,
            result: StartBusListenerResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_bus_listener() {
        let serialized = [7, 0, 0, 0, 41, 1, 1];

        let msg = StartBusListenerReply {
            serial: 1,
            result: StartBusListenerResult::InvalidBusListener,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn already_started() {
        let serialized = [7, 0, 0, 0, 41, 1, 2];

        let msg = StartBusListenerReply {
            serial: 1,
            result: StartBusListenerResult::AlreadyStarted,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StartBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
