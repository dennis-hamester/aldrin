use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ObjectCookie;
use crate::serialized_value::SerializedValueSlice;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum CreateObjectReplyKind {
    Ok = 0,
    DuplicateObject = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum CreateObjectResult {
    Ok(ObjectCookie),
    DuplicateObject,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

impl MessageOps for CreateObjectReply {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateObjectReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateObjectReply);

        serializer.put_varint_u32_le(self.serial);

        match self.result {
            CreateObjectResult::Ok(cookie) => {
                serializer.put_discriminant_u8(CreateObjectReplyKind::Ok);
                serializer.put_uuid(cookie.0);
            }

            CreateObjectResult::DuplicateObject => {
                serializer.put_discriminant_u8(CreateObjectReplyKind::DuplicateObject);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateObjectReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let result = match deserializer.try_get_discriminant_u8()? {
            CreateObjectReplyKind::Ok => {
                let cookie = deserializer.try_get_uuid().map(ObjectCookie)?;
                CreateObjectResult::Ok(cookie)
            }

            CreateObjectReplyKind::DuplicateObject => CreateObjectResult::DuplicateObject,
        };

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }
}

impl Sealed for CreateObjectReply {}

impl From<CreateObjectReply> for Message {
    fn from(msg: CreateObjectReply) -> Self {
        Self::CreateObjectReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{CreateObjectReply, CreateObjectResult};
    use crate::ids::ObjectCookie;
    use uuid::uuid;

    #[test]
    fn ok() {
        let serialized = [
            23, 0, 0, 0, 4, 1, 0, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = CreateObjectReply {
            serial: 1,
            result: CreateObjectResult::Ok(ObjectCookie(uuid!(
                "b7c3be13-5377-466e-b4bf-373876523d1b"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn duplicate_object() {
        let serialized = [7, 0, 0, 0, 4, 1, 1];

        let msg = CreateObjectReply {
            serial: 1,
            result: CreateObjectResult::DuplicateObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateObjectReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
