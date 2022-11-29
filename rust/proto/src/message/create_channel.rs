use super::message_ops::Sealed;
use super::{
    ChannelEnd, Message, MessageKind, MessageOps, MessageSerializer,
    MessageWithoutValueDeserializer,
};
use crate::error::{DeserializeError, SerializeError};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CreateChannel {
    pub serial: u32,
    pub claim: ChannelEnd,
}

impl MessageOps for CreateChannel {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateChannel
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateChannel);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_discriminant_u8(self.claim);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateChannel)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let claim = deserializer.try_get_discriminant_u8()?;

        deserializer.finish()?;
        Ok(Self { serial, claim })
    }

    fn value_opt(&self) -> Option<&[u8]> {
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
    use super::super::{ChannelEnd, Message};
    use super::CreateChannel;

    #[test]
    fn sender() {
        let serialized = [7, 0, 0, 0, 31, 1, 0];

        let msg = CreateChannel {
            serial: 1,
            claim: ChannelEnd::Sender,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateChannel(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn receiver() {
        let serialized = [7, 0, 0, 0, 31, 1, 1];

        let msg = CreateChannel {
            serial: 1,
            claim: ChannelEnd::Receiver,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateChannel(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
