use super::message_ops::Sealed;
use super::{MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ChannelCookie;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CreateChannelReply {
    pub serial: u32,
    pub cookie: ChannelCookie,
}

impl MessageOps for CreateChannelReply {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateChannelReply
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateChannelReply);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateChannelReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;

        deserializer.finish()?;
        Ok(Self { serial, cookie })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for CreateChannelReply {}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::CreateChannelReply;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn create_channel_reply() {
        let serialized = [
            32, 1, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a, 0xd9, 0xdd,
            0xcd, 0x7e, 0x72,
        ];

        let msg = CreateChannelReply {
            serial: 1,
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateChannelReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
