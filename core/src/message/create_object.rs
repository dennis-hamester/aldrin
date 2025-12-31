use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{ObjectUuid, SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: ObjectUuid,
}

impl MessageOps for CreateObject {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateObject
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateObject);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.uuid.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateObject)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let uuid = deserializer.try_get_uuid().map(ObjectUuid)?;

        deserializer.finish()?;
        Ok(Self { serial, uuid })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for CreateObject {}

impl From<CreateObject> for Message {
    fn from(msg: CreateObject) -> Self {
        Self::CreateObject(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::CreateObject;
    use crate::ObjectUuid;
    use uuid::uuid;

    #[test]
    fn create_object() {
        let serialized = [
            22, 0, 0, 0, 3, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = CreateObject {
            serial: 1,
            uuid: ObjectUuid(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateObject(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
