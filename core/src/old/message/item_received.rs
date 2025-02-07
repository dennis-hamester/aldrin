use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::SerializeError;
use crate::ids::ChannelCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_serializer::Serialize;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ItemReceived {
    pub cookie: ChannelCookie,
    pub value: SerializedValue,
}

impl ItemReceived {
    pub fn with_serialize_value<T: Serialize + ?Sized>(
        cookie: ChannelCookie,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(value)?;
        Ok(Self { cookie, value })
    }
}

impl MessageOps for ItemReceived {
    fn kind(&self) -> MessageKind {
        MessageKind::ItemReceived
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::ItemReceived)?;

        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::ItemReceived)?;

        let cookie = deserializer.try_get_uuid().map(ChannelCookie)?;
        let value = deserializer.finish()?;

        Ok(Self { cookie, value })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }
}

impl Sealed for ItemReceived {}

impl From<ItemReceived> for Message {
    fn from(msg: ItemReceived) -> Self {
        Self::ItemReceived(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::ItemReceived;
    use crate::ids::ChannelCookie;
    use uuid::uuid;

    #[test]
    fn item_received() {
        let serialized = [
            27, 0, 0, 0, 28, 2, 0, 0, 0, 3, 4, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb,
        ];
        let value = 4u8;

        let msg = ItemReceived::with_serialize_value(
            ChannelCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            &value,
        )
        .unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ItemReceived(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
