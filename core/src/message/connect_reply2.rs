use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, SerializedValue,
    SerializedValueSlice, Serializer,
};
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

    pub fn serialize_user_as<T: Tag>(
        &mut self,
        user: impl Serialize<T>,
    ) -> Result<&mut Self, SerializeError> {
        self.user = SerializedValue::serialize_as(user).map(Some)?;
        Ok(self)
    }

    pub fn serialize_user<T: PrimaryTag + Serialize<T::Tag>>(
        &mut self,
        user: T,
    ) -> Result<&mut Self, SerializeError> {
        self.serialize_user_as(user)
    }

    pub fn deserialize_user_as<T: Tag, U: Deserialize<T>>(
        &self,
    ) -> Option<Result<U, DeserializeError>> {
        self.user
            .as_deref()
            .map(SerializedValueSlice::deserialize_as)
    }

    pub fn deserialize_user<T: PrimaryTag + Deserialize<T::Tag>>(
        &self,
    ) -> Option<Result<T, DeserializeError>> {
        self.deserialize_user_as()
    }
}

impl Tag for ConnectReplyData {}

impl PrimaryTag for ConnectReplyData {
    type Tag = Self;
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ConnectReplyDataField {
    User = 0,
}

impl Serialize<Self> for ConnectReplyData {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<ConnectReplyData> for &ConnectReplyData {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize_if_some::<tags::Option<tags::Value>>(
            ConnectReplyDataField::User,
            &self.user,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for ConnectReplyData {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut user = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ConnectReplyDataField::User) => user = deserializer.deserialize()?,
                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self { user })
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

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
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
    use super::{ConnectReply2, ConnectReplyData, ConnectResult};
    use crate::SerializedValue;

    #[test]
    fn ok() {
        let serialized = [13, 0, 0, 0, 47, 2, 0, 0, 0, 65, 0, 0, 1];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2 {
            result: ConnectResult::Ok(1),
            value: SerializedValue::serialize(&value).unwrap(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn rejected() {
        let serialized = [12, 0, 0, 0, 47, 2, 0, 0, 0, 65, 0, 1];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2 {
            result: ConnectResult::Rejected,
            value: SerializedValue::serialize(&value).unwrap(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn incompatible_version() {
        let serialized = [12, 0, 0, 0, 47, 2, 0, 0, 0, 65, 0, 2];
        let value = ConnectReplyData::new();

        let msg = ConnectReply2 {
            result: ConnectResult::IncompatibleVersion,
            value: SerializedValue::serialize(&value).unwrap(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::ConnectReply2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
