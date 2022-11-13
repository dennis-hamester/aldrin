use super::message_ops::Sealed;
use super::{MessageKind, MessageOps, MessageSerializer, MessageWithoutValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceCookie;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: ServiceCookie,
}

impl MessageOps for DestroyService {
    fn kind(&self) -> MessageKind {
        MessageKind::DestroyService
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::DestroyService);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.cookie.0);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::DestroyService)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;

        deserializer.finish()?;
        Ok(Self { serial, cookie })
    }

    fn value_buf_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl Sealed for DestroyService {}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::DestroyService;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn destroy_service() {
        let serialized = [
            14, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37, 0x38, 0x76,
            0x52, 0x3d, 0x1b,
        ];

        let msg = DestroyService {
            serial: 1,
            cookie: ServiceCookie(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::DestroyService(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
