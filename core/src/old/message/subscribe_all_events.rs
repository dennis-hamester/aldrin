use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, OptionKind};
use crate::ids::ServiceCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeAllEvents {
    pub serial: Option<u32>,
    pub service_cookie: ServiceCookie,
}

impl MessageOps for SubscribeAllEvents {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeAllEvents
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeAllEvents);

        match self.serial {
            None => {
                serializer.put_discriminant_u8(OptionKind::None);
            }

            Some(serial) => {
                serializer.put_discriminant_u8(OptionKind::Some);
                serializer.put_varint_u32_le(serial);
            }
        }

        serializer.put_uuid(self.service_cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeAllEvents)?;

        let serial = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            service_cookie,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for SubscribeAllEvents {}

impl From<SubscribeAllEvents> for Message {
    fn from(msg: SubscribeAllEvents) -> Self {
        Self::SubscribeAllEvents(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeAllEvents;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn no_serial() {
        let serialized = [
            22, 0, 0, 0, 58, 0, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc, 0x4e,
            0x0e, 0x84, 0x80, 0xdf, 0xad,
        ];

        let msg = SubscribeAllEvents {
            serial: None,
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeAllEvents(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn serial() {
        let serialized = [
            23, 0, 0, 0, 58, 1, 2, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc,
            0x4e, 0x0e, 0x84, 0x80, 0xdf, 0xad,
        ];

        let msg = SubscribeAllEvents {
            serial: Some(2),
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeAllEvents(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
