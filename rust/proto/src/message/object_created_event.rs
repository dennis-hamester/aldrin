use super::message_ops::Sealed;
use super::{
    Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer,
    OptionKind,
};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::{ObjectCookie, ObjectId, ObjectUuid};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ObjectCreatedEvent {
    pub id: ObjectId,
    pub serial: Option<u32>,
}

impl MessageOps for ObjectCreatedEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::ObjectCreatedEvent
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ObjectCreatedEvent);

        serializer.put_uuid(self.id.uuid.0);
        serializer.put_uuid(self.id.cookie.0);

        match self.serial {
            None => {
                serializer.put_discriminant_u8(OptionKind::None);
            }

            Some(serial) => {
                serializer.put_discriminant_u8(OptionKind::Some);
                serializer.put_varint_u32_le(serial);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ObjectCreatedEvent)?;

        let uuid = deserializer.try_get_uuid().map(ObjectUuid)?;
        let cookie = deserializer.try_get_uuid().map(ObjectCookie)?;

        let serial = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        deserializer.finish()?;
        Ok(Self {
            id: ObjectId { uuid, cookie },
            serial,
        })
    }

    fn value_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for ObjectCreatedEvent {}

impl From<ObjectCreatedEvent> for Message {
    fn from(msg: ObjectCreatedEvent) -> Self {
        Self::ObjectCreatedEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ObjectCreatedEvent;
    use crate::ids::{ObjectCookie, ObjectId, ObjectUuid};
    use uuid::uuid;

    #[test]
    fn no_serial() {
        let serialized = [
            38, 0, 0, 0, 10, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29,
            0xa0, 0x11, 0x32, 0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac,
            0x74, 0xce, 0xbf, 0x53, 0xae, 0xbb, 0x10, 0,
        ];

        let msg = ObjectCreatedEvent {
            id: ObjectId {
                uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
            },
            serial: None,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ObjectCreatedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn serial() {
        let serialized = [
            39, 0, 0, 0, 10, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29,
            0xa0, 0x11, 0x32, 0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac,
            0x74, 0xce, 0xbf, 0x53, 0xae, 0xbb, 0x10, 1, 2,
        ];

        let msg = ObjectCreatedEvent {
            id: ObjectId {
                uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
            },
            serial: Some(2),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ObjectCreatedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
