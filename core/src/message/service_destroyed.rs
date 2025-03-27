use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice, ServiceCookie};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ServiceDestroyed {
    pub service_cookie: ServiceCookie,
}

impl MessageOps for ServiceDestroyed {
    fn kind(&self) -> MessageKind {
        MessageKind::ServiceDestroyed
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::ServiceDestroyed);

        serializer.put_uuid(self.service_cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::ServiceDestroyed)?;

        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        deserializer.finish()?;
        Ok(Self { service_cookie })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for ServiceDestroyed {}

impl From<ServiceDestroyed> for Message {
    fn from(msg: ServiceDestroyed) -> Self {
        Self::ServiceDestroyed(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::ServiceDestroyed;
    use crate::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn service_destroyed_event() {
        let serialized = [
            21, 0, 0, 0, 32, 0x54, 0x37, 0xe9, 0x74, 0xe5, 0x91, 0x48, 0x23, 0xb2, 0xcd, 0x89,
            0xda, 0x0d, 0xda, 0x17, 0x79,
        ];

        let msg = ServiceDestroyed {
            service_cookie: ServiceCookie(uuid!("5437e974-e591-4823-b2cd-89da0dda1779")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::ServiceDestroyed(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
