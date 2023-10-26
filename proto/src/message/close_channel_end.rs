use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageDeserializeError, MessageKind, MessageOps,
    MessageWithoutValueDeserializer,
};
use crate::ids::ChannelCookie;
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CloseChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

impl MessageOps for CloseChannelEnd {
    fn kind(&self) -> MessageKind {
        MessageKind::CloseChannelEnd
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CloseChannelEnd);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.end);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CloseChannelEnd)?;

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

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for CloseChannelEnd {}

impl From<CloseChannelEnd> for Message {
    fn from(msg: CloseChannelEnd) -> Self {
        Self::CloseChannelEnd(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEnd, Message};
    use super::CloseChannelEnd;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            23, 0, 0, 0, 21, 1, 0xde, 0xcf, 0x4b, 0x2f, 0x56, 0x2c, 0x4c, 0x1b, 0xb8, 0x84, 0x61,
            0x47, 0xe3, 0xde, 0x76, 0xc0, 0,
        ];

        let msg = CloseChannelEnd {
            serial: 1,
            cookie: ChannelCookie(uuid!("decf4b2f-562c-4c1b-b884-6147e3de76c0")),
            end: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CloseChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [
            23, 0, 0, 0, 21, 1, 0xde, 0xcf, 0x4b, 0x2f, 0x56, 0x2c, 0x4c, 0x1b, 0xb8, 0x84, 0x61,
            0x47, 0xe3, 0xde, 0x76, 0xc0, 1,
        ];

        let msg = CloseChannelEnd {
            serial: 1,
            cookie: ChannelCookie(uuid!("decf4b2f-562c-4c1b-b884-6147e3de76c0")),
            end: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CloseChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
