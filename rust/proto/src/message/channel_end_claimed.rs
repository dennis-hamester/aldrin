use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageKind, MessageOps, MessageSerializer,
    MessageWithoutValueDeserializer,
};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ChannelCookie;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChannelEndClaimed {
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

impl MessageOps for ChannelEndClaimed {
    fn kind(&self) -> MessageKind {
        MessageKind::ChannelEndClaimed
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ChannelEndClaimed);

        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.end);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ChannelEndClaimed)?;

        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;
        let end = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { cookie, end })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for ChannelEndClaimed {}

impl From<ChannelEndClaimed> for Message {
    fn from(msg: ChannelEndClaimed) -> Self {
        Self::ChannelEndClaimed(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEnd, Message};
    use super::ChannelEndClaimed;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            38, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a, 0xd9, 0xdd, 0xcd,
            0x7e, 0x72, 0,
        ];

        let msg = ChannelEndClaimed {
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ChannelEndClaimed(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [
            38, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a, 0xd9, 0xdd, 0xcd,
            0x7e, 0x72, 1,
        ];

        let msg = ChannelEndClaimed {
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ChannelEndClaimed(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
