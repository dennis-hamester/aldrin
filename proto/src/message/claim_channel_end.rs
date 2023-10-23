use super::message_ops::Sealed;
use super::{
    ChannelEnd, ChannelEndWithCapacity, Message, MessageDeserializeError, MessageKind, MessageOps,
    MessageSerializeError, MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ChannelCookie;
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ClaimChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEndWithCapacity,
}

impl MessageOps for ClaimChannelEnd {
    fn kind(&self) -> MessageKind {
        MessageKind::ClaimChannelEnd
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ClaimChannelEnd);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        match self.end {
            ChannelEndWithCapacity::Sender => serializer.put_discriminant_u8(ChannelEnd::Sender),
            ChannelEndWithCapacity::Receiver(capacity) => {
                serializer.put_discriminant_u8(ChannelEnd::Receiver);
                serializer.put_varint_u32_le(capacity);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ClaimChannelEnd)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;

        let end = match deserializer.try_get_discriminant_u8()? {
            ChannelEnd::Sender => ChannelEndWithCapacity::Sender,
            ChannelEnd::Receiver => {
                let capacity = deserializer.try_get_varint_u32_le()?;
                ChannelEndWithCapacity::Receiver(capacity)
            }
        };

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

impl Sealed for ClaimChannelEnd {}

impl From<ClaimChannelEnd> for Message {
    fn from(msg: ClaimChannelEnd) -> Self {
        Self::ClaimChannelEnd(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::{ChannelEndWithCapacity, Message};
    use super::ClaimChannelEnd;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn sender() {
        let serialized = [
            23, 0, 0, 0, 24, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 0,
        ];

        let msg = ClaimChannelEnd {
            serial: 0,
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEndWithCapacity::Sender,
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
            24, 0, 0, 0, 24, 0, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 1, 16,
        ];

        let msg = ClaimChannelEnd {
            serial: 0,
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            end: ChannelEndWithCapacity::Receiver(16),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ClaimChannelEnd(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
