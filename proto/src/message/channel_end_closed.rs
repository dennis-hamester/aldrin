use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ChannelCookie;
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ChannelEndClosed {
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

impl MessageOps for ChannelEndClosed {
    fn kind(&self) -> MessageKind {
        MessageKind::ChannelEndClosed
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ChannelEndClosed);

        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.end);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ChannelEndClosed)?;

        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;
        let end = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { cookie, end })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for ChannelEndClosed {}

impl From<ChannelEndClosed> for Message {
    fn from(msg: ChannelEndClosed) -> Self {
        Self::ChannelEndClosed(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEnd, Message};
    use super::ChannelEndClosed;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            22, 0, 0, 0, 23, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 0,
        ];

        let msg = ChannelEndClosed {
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ChannelEndClosed(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [
            22, 0, 0, 0, 23, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 1,
        ];

        let msg = ChannelEndClosed {
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ChannelEndClosed(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
