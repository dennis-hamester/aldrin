use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::{ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ServiceDestroyedEvent {
    pub id: ServiceId,
}

impl MessageOps for ServiceDestroyedEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::ServiceDestroyedEvent
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ServiceDestroyedEvent);

        serializer.put_uuid(self.id.object_id.uuid.0);
        serializer.put_uuid(self.id.object_id.cookie.0);
        serializer.put_uuid(self.id.uuid.0);
        serializer.put_uuid(self.id.cookie.0);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ServiceDestroyedEvent)?;

        let object_uuid = deserializer.try_get_uuid().map(ObjectUuid)?;
        let object_cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
        let uuid = deserializer.try_get_uuid().map(ServiceUuid)?;
        let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

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
        })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for ServiceDestroyedEvent {}

impl From<ServiceDestroyedEvent> for Message {
    fn from(msg: ServiceDestroyedEvent) -> Self {
        Self::ServiceDestroyedEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ServiceDestroyedEvent;
    use crate::ids::{ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn service_destroyed_event() {
        let serialized = [
            20, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29, 0xa0, 0x11, 0x32,
            0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac, 0x74, 0xce, 0xbf,
            0x53, 0xae, 0xbb, 0x10, 0xfc, 0x2d, 0x4c, 0x9e, 0xe0, 0x5b, 0x4a, 0x31, 0x90, 0xf8,
            0x81, 0x90, 0x80, 0x91, 0xd0, 0x28, 0x54, 0x37, 0xe9, 0x74, 0xe5, 0x91, 0x48, 0x23,
            0xb2, 0xcd, 0x89, 0xda, 0x0d, 0xda, 0x17, 0x79,
        ];

        let msg = ServiceDestroyedEvent {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                    cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
                },
                uuid: ServiceUuid(uuid!("fc2d4c9e-e05b-4a31-90f8-81908091d028")),
                cookie: ServiceCookie(uuid!("5437e974-e591-4823-b2cd-89da0dda1779")),
            },
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ServiceDestroyedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
