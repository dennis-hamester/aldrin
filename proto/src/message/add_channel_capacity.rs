use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageWithoutValueDeserializer,
};
use crate::ids::ChannelCookie;
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct AddChannelCapacity {
    pub cookie: ChannelCookie,
    pub capacity: u32,
}

impl MessageOps for AddChannelCapacity {
    fn kind(&self) -> MessageKind {
        MessageKind::AddChannelCapacity
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::AddChannelCapacity);

        serializer.put_uuid(self.cookie.0);
        serializer.put_varint_u32_le(self.capacity);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::AddChannelCapacity)?;

        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;
        let capacity = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self { cookie, capacity })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for AddChannelCapacity {}

impl From<AddChannelCapacity> for Message {
    fn from(msg: AddChannelCapacity) -> Self {
        Self::AddChannelCapacity(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::AddChannelCapacity;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn add_channel_capacity() {
        let serialized = [
            22, 0, 0, 0, 29, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 16,
        ];

        let msg = AddChannelCapacity {
            cookie: ChannelCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            capacity: 16,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddChannelCapacity(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
