use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::{ObjectCookie, ObjectId, ObjectUuid};
use crate::value::SerializedValue;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ObjectDestroyedEvent {
    pub id: ObjectId,
}

impl MessageOps for ObjectDestroyedEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::ObjectDestroyedEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ObjectDestroyedEvent);

        serializer.put_uuid(self.id.uuid.0);
        serializer.put_uuid(self.id.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ObjectDestroyedEvent)?;

        let uuid = deserializer.try_get_uuid().map(ObjectUuid)?;
        let cookie = deserializer.try_get_uuid().map(ObjectCookie)?;

        deserializer.finish()?;
        Ok(Self {
            id: ObjectId { uuid, cookie },
        })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for ObjectDestroyedEvent {}

impl From<ObjectDestroyedEvent> for Message {
    fn from(msg: ObjectDestroyedEvent) -> Self {
        Self::ObjectDestroyedEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ObjectDestroyedEvent;
    use crate::ids::{ObjectCookie, ObjectId, ObjectUuid};
    use uuid::uuid;

    #[test]
    fn object_destroyed_event() {
        let serialized = [
            37, 0, 0, 0, 11, 0x31, 0x0a, 0xce, 0xa9, 0x6d, 0x9e, 0x4f, 0x80, 0xad, 0x7d, 0x29,
            0xa0, 0x11, 0x32, 0xdb, 0xc3, 0xb1, 0x53, 0x47, 0x65, 0x0e, 0xdd, 0x45, 0xbd, 0xac,
            0x74, 0xce, 0xbf, 0x53, 0xae, 0xbb, 0x10,
        ];

        let msg = ObjectDestroyedEvent {
            id: ObjectId {
                uuid: ObjectUuid(uuid!("310acea9-6d9e-4f80-ad7d-29a01132dbc3")),
                cookie: ObjectCookie(uuid!("b1534765-0edd-45bd-ac74-cebf53aebb10")),
            },
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ObjectDestroyedEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
