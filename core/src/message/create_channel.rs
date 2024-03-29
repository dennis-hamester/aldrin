use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::channel_end::{ChannelEnd, ChannelEndWithCapacity};
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateChannel {
    pub serial: u32,
    pub end: ChannelEndWithCapacity,
}

impl MessageOps for CreateChannel {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateChannel
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateChannel);

        serializer.put_varint_u32_le(self.serial);

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
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateChannel)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let end = match deserializer.try_get_discriminant_u8()? {
            ChannelEnd::Sender => ChannelEndWithCapacity::Sender,
            ChannelEnd::Receiver => {
                let capacity = deserializer.try_get_varint_u32_le()?;
                ChannelEndWithCapacity::Receiver(capacity)
            }
        };

        deserializer.finish()?;
        Ok(Self { serial, end })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for CreateChannel {}

impl From<CreateChannel> for Message {
    fn from(msg: CreateChannel) -> Self {
        Self::CreateChannel(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::CreateChannel;
    use crate::channel_end::ChannelEndWithCapacity;

    #[test]
    fn sender() {
        let serialized = [7, 0, 0, 0, 19, 1, 0];

        let msg = CreateChannel {
            serial: 1,
            end: ChannelEndWithCapacity::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateChannel(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [8, 0, 0, 0, 19, 1, 1, 16];

        let msg = CreateChannel {
            serial: 1,
            end: ChannelEndWithCapacity::Receiver(16),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateChannel(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
