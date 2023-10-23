use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer, OptionKind,
};
use crate::ids::ServiceCookie;
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SubscribeEvent {
    pub serial: Option<u32>,
    pub service_cookie: ServiceCookie,
    pub event: u32,
}

impl MessageOps for SubscribeEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::SubscribeEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::SubscribeEvent);

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
        serializer.put_varint_u32_le(self.event);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::SubscribeEvent)?;

        let serial = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
        let event = deserializer.try_get_varint_u32_le()?;

        deserializer.finish()?;
        Ok(Self {
            serial,
            service_cookie,
            event,
        })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for SubscribeEvent {}

impl From<SubscribeEvent> for Message {
    fn from(msg: SubscribeEvent) -> Self {
        Self::SubscribeEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::SubscribeEvent;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn no_serial() {
        let serialized = [
            23, 0, 0, 0, 13, 0, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc, 0x4e,
            0x0e, 0x84, 0x80, 0xdf, 0xad, 1,
        ];

        let msg = SubscribeEvent {
            serial: None,
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
            event: 1,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn serial() {
        let serialized = [
            24, 0, 0, 0, 13, 1, 2, 0x94, 0x5f, 0xc6, 0xe4, 0xe8, 0x9c, 0x49, 0x61, 0xb7, 0xbc,
            0x4e, 0x0e, 0x84, 0x80, 0xdf, 0xad, 3,
        ];

        let msg = SubscribeEvent {
            serial: Some(2),
            service_cookie: ServiceCookie(uuid!("945fc6e4-e89c-4961-b7bc-4e0e8480dfad")),
            event: 3,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::SubscribeEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
