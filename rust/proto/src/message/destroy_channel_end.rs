use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ChannelCookie;
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct DestroyChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

impl MessageOps for DestroyChannelEnd {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyChannelEnd
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyChannelEnd);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);
        serializer.put_discriminant_u8(self.end);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyChannelEnd)?;

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

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for DestroyChannelEnd {}

impl From<DestroyChannelEnd> for Message {
    fn from(msg: DestroyChannelEnd) -> Self {
        Self::DestroyChannelEnd(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEnd, Message};
    use super::DestroyChannelEnd;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            23, 0, 0, 0, 33, 1, 0xde, 0xcf, 0x4b, 0x2f, 0x56, 0x2c, 0x4c, 0x1b, 0xb8, 0x84, 0x61,
            0x47, 0xe3, 0xde, 0x76, 0xc0, 0,
        ];

        let msg = DestroyChannelEnd {
            serial: 1,
            cookie: ChannelCookie(uuid!("decf4b2f-562c-4c1b-b884-6147e3de76c0")),
            end: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [
            23, 0, 0, 0, 33, 1, 0xde, 0xcf, 0x4b, 0x2f, 0x56, 0x2c, 0x4c, 0x1b, 0xb8, 0x84, 0x61,
            0x47, 0xe3, 0xde, 0x76, 0xc0, 1,
        ];

        let msg = DestroyChannelEnd {
            serial: 1,
            cookie: ChannelCookie(uuid!("decf4b2f-562c-4c1b-b884-6147e3de76c0")),
            end: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
