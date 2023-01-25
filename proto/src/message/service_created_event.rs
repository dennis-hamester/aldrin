use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer, OptionKind,
};
use crate::ids::{ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid};
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ServiceCreatedEvent {
    pub id: ServiceId,
    pub serial: Option<u32>,
}

impl MessageOps for ServiceCreatedEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::ServiceCreatedEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ServiceCreatedEvent);

        serializer.put_uuid(self.id.object_id.uuid.0);
        serializer.put_uuid(self.id.object_id.cookie.0);
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

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ServiceCreatedEvent)?;

        let object_uuid = deserializer.try_get_uuid().map(ObjectUuid)?;
        let object_cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
        let uuid = deserializer.try_get_uuid().map(ServiceUuid)?;
        let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        let serial = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        deserializer.finish()?;
        Ok(Self {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: object_uuid,
                    cookie: object_cookie,
                },
                uuid,
                cookie,
            },
            serial,
        })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for ServiceCreatedEvent {}

impl From<ServiceCreatedEvent> for Message {
    fn from(msg: ServiceCreatedEvent) -> Self {
        Self::ServiceCreatedEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ServiceCreatedEvent;
    use crate::ids::{ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn no_serial() {
        let serialized = [
            70, 0, 0, 0, 19, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29,
            0xa0, 0x11, 0x32, 0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac,
            0x74, 0xce, 0xbf, 0x53, 0xae, 0xbb, 0x10, 0xfc, 0x2d, 0x4c, 0x9e, 0xe0, 0x5b, 0x4a,
            0x31, 0x90, 0xf8, 0x81, 0x90, 0x80, 0x91, 0xd0, 0x28, 0x54, 0x37, 0xe9, 0x74, 0xe5,
            0x91, 0x48, 0x23, 0xb2, 0xcd, 0x89, 0xda, 0x0d, 0xda, 0x17, 0x79, 0,
        ];

        let msg = ServiceCreatedEvent {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                    cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
                },
                uuid: ServiceUuid(uuid!("fc2d4c9e-e05b-4a31-90f8-81908091d028")),
                cookie: ServiceCookie(uuid!("5437e974-e591-4823-b2cd-89da0dda1779")),
            },
            serial: None,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ServiceCreatedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn serial() {
        let serialized = [
            71, 0, 0, 0, 19, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29,
            0xa0, 0x11, 0x32, 0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac,
            0x74, 0xce, 0xbf, 0x53, 0xae, 0xbb, 0x10, 0xfc, 0x2d, 0x4c, 0x9e, 0xe0, 0x5b, 0x4a,
            0x31, 0x90, 0xf8, 0x81, 0x90, 0x80, 0x91, 0xd0, 0x28, 0x54, 0x37, 0xe9, 0x74, 0xe5,
            0x91, 0x48, 0x23, 0xb2, 0xcd, 0x89, 0xda, 0x0d, 0xda, 0x17, 0x79, 1, 2,
        ];

        let msg = ServiceCreatedEvent {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                    cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
                },
                uuid: ServiceUuid(uuid!("fc2d4c9e-e05b-4a31-90f8-81908091d028")),
                cookie: ServiceCookie(uuid!("5437e974-e591-4823-b2cd-89da0dda1779")),
            },
            serial: Some(2),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ServiceCreatedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
