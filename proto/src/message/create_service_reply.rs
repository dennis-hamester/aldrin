use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::ids::ServiceCookie;
use crate::serialized_value::SerializedValue;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum CreateServiceReplyKind {
    Ok = 0,
    DuplicateService = 1,
    InvalidObject = 2,
    ForeignObject = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum CreateServiceResult {
    Ok(ServiceCookie),
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

impl MessageOps for CreateServiceReply {
    fn kind(&self) -> MessageKind {
        MessageKind::CreateServiceReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::CreateServiceReply);

        serializer.put_varint_u32_le(self.serial);

        match self.result {
            CreateServiceResult::Ok(cookie) => {
                serializer.put_discriminant_u8(CreateServiceReplyKind::Ok);
                serializer.put_uuid(cookie.0);
            }

            CreateServiceResult::DuplicateService => {
                serializer.put_discriminant_u8(CreateServiceReplyKind::DuplicateService);
            }

            CreateServiceResult::InvalidObject => {
                serializer.put_discriminant_u8(CreateServiceReplyKind::InvalidObject);
            }

            CreateServiceResult::ForeignObject => {
                serializer.put_discriminant_u8(CreateServiceReplyKind::ForeignObject);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::CreateServiceReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        let result = match deserializer.try_get_discriminant_u8()? {
            CreateServiceReplyKind::Ok => {
                let cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
                CreateServiceResult::Ok(cookie)
            }

            CreateServiceReplyKind::DuplicateService => CreateServiceResult::DuplicateService,
            CreateServiceReplyKind::InvalidObject => CreateServiceResult::InvalidObject,
            CreateServiceReplyKind::ForeignObject => CreateServiceResult::ForeignObject,
        };

        deserializer.finish()?;
        Ok(Self { serial, result })
    }

    fn value(&self) -> Option<&SerializedValue> {
        None
    }
}

impl Sealed for CreateServiceReply {}

impl From<CreateServiceReply> for Message {
    fn from(msg: CreateServiceReply) -> Self {
        Self::CreateServiceReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::{CreateServiceReply, CreateServiceResult};
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn ok() {
        let serialized = [
            23, 0, 0, 0, 8, 1, 0, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = CreateServiceReply {
            serial: 1,
            result: CreateServiceResult::Ok(ServiceCookie(uuid!(
                "b7c3be13-5377-466e-b4bf-373876523d1b"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn duplicate_service() {
        let serialized = [7, 0, 0, 0, 8, 1, 1];

        let msg = CreateServiceReply {
            serial: 1,
            result: CreateServiceResult::DuplicateService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_object() {
        let serialized = [7, 0, 0, 0, 8, 1, 2];

        let msg = CreateServiceReply {
            serial: 1,
            result: CreateServiceResult::InvalidObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn foreign_object() {
        let serialized = [7, 0, 0, 0, 8, 1, 3];

        let msg = CreateServiceReply {
            serial: 1,
            result: CreateServiceResult::ForeignObject,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CreateServiceReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
