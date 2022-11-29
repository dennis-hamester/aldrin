use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageKind, MessageOps, MessageSerializer,
    MessageWithoutValueDeserializer,
};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ChannelCookie;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClaimChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

impl MessageOps for ClaimChannelEnd {
    fn kind(&self) -> MessageKind {
        MessageKind::ClaimChannelEnd
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ClaimChannelEnd);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.end);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ClaimChannelEnd)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;
        let end = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            cookie,
            end,
        })
    }

    fn value_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for ClaimChannelEnd {}

impl From<ClaimChannelEnd> for Message {
    fn from(msg: ClaimChannelEnd) -> Self {
        Self::ClaimChannelEnd(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEnd, Message};
    use super::ClaimChannelEnd;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            23, 0, 0, 0, 36, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 0,
        ];

        let msg = ClaimChannelEnd {
            serial: 0,
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [
            23, 0, 0, 0, 36, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 1,
        ];

        let msg = ClaimChannelEnd {
            serial: 0,
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
