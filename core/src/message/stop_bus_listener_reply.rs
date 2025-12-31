use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum StopBusListenerResult {
    Ok = 0,
    InvalidBusListener = 1,
    NotStarted = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct StopBusListenerReply {
    pub serial: u32,
    pub result: StopBusListenerResult,
}

impl MessageOps for StopBusListenerReply {
    fn kind(&self) -> MessageKind {
        MessageKind::StopBusListenerReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::StopBusListenerReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.result);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::StopBusListenerReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let result = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for StopBusListenerReply {}

impl From<StopBusListenerReply> for Message {
    fn from(msg: StopBusListenerReply) -> Self {
        Self::StopBusListenerReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::{StopBusListenerReply, StopBusListenerResult};

    #[test]
    fn ok() {
        let serialized = [7, 0, 0, 0, 43, 1, 0];

        let msg = StopBusListenerReply {
            serial: 1,
            result: StopBusListenerResult::Ok,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StopBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_bus_listener() {
        let serialized = [7, 0, 0, 0, 43, 1, 1];

        let msg = StopBusListenerReply {
            serial: 1,
            result: StopBusListenerResult::InvalidBusListener,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StopBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn not_started() {
        let serialized = [7, 0, 0, 0, 43, 1, 2];

        let msg = StopBusListenerReply {
            serial: 1,
            result: StopBusListenerResult::NotStarted,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::StopBusListenerReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
