use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::{DeserializeError, SerializeError};
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{AsSerializeArg, Serialize, Serializer};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ConnectReplyData {
    pub user: Option<SerializedValue>,
}

impl ConnectReplyData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize_user<T: Serialize + ?Sized>(
        &mut self,
        user: &T,
    ) -> Result<&mut Self, SerializeError> {
        self.user = SerializedValue::serialize(user).map(Some)?;
        Ok(self)
    }

    pub fn deserialize_user<T: Deserialize>(&self) -> Option<Result<T, DeserializeError>> {
        self.user.as_deref().map(SerializedValueSlice::deserialize)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ConnectReplyDataField {
    User = 0,
}

impl Serialize for ConnectReplyData {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(1)?;
        serializer.serialize_field(ConnectReplyDataField::User, &self.user)?;
        serializer.finish()
    }
}

impl Deserialize for ConnectReplyData {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut user = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            let Ok(field) = deserializer.try_id() else {
                deserializer.skip()?;
                continue;
            };

            match field {
                ConnectReplyDataField::User => user = deserializer.deserialize()?,
            }
        }

        deserializer.finish(Self { user })
    }
}

impl AsSerializeArg for ConnectReplyData {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum ConnectReplyKind {
    Ok = 0,
    Rejected = 1,
    IncompatibleVersion = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum ConnectResult {
    Ok(u32),
    Rejected,
    IncompatibleVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ConnectReply2 {
    pub result: ConnectResult,
    pub value: SerializedValue,
}

impl ConnectReply2 {
    pub fn ok_with_serialize_data(
        version: u32,
        data: &ConnectReplyData,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(data)?;

        Ok(Self {
            result: ConnectResult::Ok(version),
            value,
        })
    }

    pub fn rejected_with_serialize_data(data: &ConnectReplyData) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(data)?;

        Ok(Self {
            result: ConnectResult::Rejected,
            value,
        })
    }

    pub fn incompatible_version_with_serialize_data(
        data: &ConnectReplyData,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(data)?;

        Ok(Self {
            result: ConnectResult::IncompatibleVersion,
            value,
        })
    }

    pub fn deserialize_connect_data(&self) -> Result<ConnectReplyData, DeserializeError> {
        self.value.deserialize()
    }
}

impl MessageOps for ConnectReply2 {
    fn kind(&self) -> MessageKind {
        MessageKind::ConnectReply2
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::ConnectReply2)?;

        match self.result {
            ConnectResult::Ok(version) => {
                serializer.put_discriminant_u8(ConnectReplyKind::Ok);
                serializer.put_varint_u32_le(version);
                serializer.finish()
            }

            ConnectResult::Rejected => {
                serializer.put_discriminant_u8(ConnectReplyKind::Rejected);
                serializer.finish()
            }

            ConnectResult::IncompatibleVersion => {
                serializer.put_discriminant_u8(ConnectReplyKind::IncompatibleVersion);
                serializer.finish()
            }
        }
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::ConnectReply2)?;

        match deserializer.try_get_discriminant_u8()? {
            ConnectReplyKind::Ok => {
                let version = deserializer.try_get_varint_u32_le()?;
                let value = deserializer.finish()?;

                Ok(Self {
                    result: ConnectResult::Ok(version),
                    value,
                })
            }

            ConnectReplyKind::Rejected => {
                let value = deserializer.finish()?;

                Ok(Self {
                    result: ConnectResult::Rejected,
                    value,
                })
            }

            ConnectReplyKind::IncompatibleVersion => {
                let value = deserializer.finish()?;

                Ok(Self {
                    result: ConnectResult::IncompatibleVersion,
                    value,
                })
            }
        }
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }
}

impl Sealed for ConnectReply2 {}

impl From<ConnectReply2> for Message {
    fn from(msg: ConnectReply2) -> Self {
        Self::ConnectReply2(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::{ConnectReply2, ConnectReplyData};

    #[test]
    fn ok() {
        let serialized = [15, 0, 0, 0, 47, 4, 0, 0, 0, 39, 1, 0, 0, 0, 1];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2::ok_with_serialize_data(1, &value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn rejected() {
        let serialized = [14, 0, 0, 0, 47, 4, 0, 0, 0, 39, 1, 0, 0, 1];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2::rejected_with_serialize_data(&value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn incompatible_version() {
        let serialized = [14, 0, 0, 0, 47, 4, 0, 0, 0, 39, 1, 0, 0, 2];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2::incompatible_version_with_serialize_data(&value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
