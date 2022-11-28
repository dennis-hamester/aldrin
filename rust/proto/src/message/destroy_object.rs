use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ObjectCookie;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: ObjectCookie,
}

impl MessageOps for DestroyObject {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyObject
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyObject);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyObject)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ObjectCookie)?;

        deserializer.finish()?;
        Ok(Self { serial, cookie })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for DestroyObject {}

impl From<DestroyObject> for Message {
    fn from(msg: DestroyObject) -> Self {
        Self::DestroyObject(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::DestroyObject;
    use crate::ids::ObjectCookie;
    use uuid::uuid;

    #[test]
    fn destroy_object() {
        let serialized = [
            22, 0, 0, 0, 5, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = DestroyObject {
            serial: 1,
            cookie: ObjectCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyObject(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
