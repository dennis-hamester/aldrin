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
pub struct ConnectData {
    pub user: Option<SerializedValue>,
}

impl ConnectData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize_user_as<T: Tag, U: Serialize<T>>(
        &mut self,
        user: U,
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

impl Tag for ConnectData {}

impl PrimaryTag for ConnectData {
    type Tag = Self;
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ConnectDataField {
    User = 0,
}

impl Serialize<Self> for ConnectData {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<ConnectData> for &ConnectData {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize_if_some::<tags::Option<tags::Value>, _>(
            ConnectDataField::User,
            &self.user,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for ConnectData {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut user = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ConnectDataField::User) => user = deserializer.deserialize()?,
                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self { user })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct Connect2 {
    pub major_version: u32,
    pub minor_version: u32,
    pub value: SerializedValue,
}

impl MessageOps for Connect2 {
    fn kind(&self) -> MessageKind {
        MessageKind::Connect2
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::Connect2)?;

        serializer.put_varint_u32_le(self.major_version);
        serializer.put_varint_u32_le(self.minor_version);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::Connect2)?;

        let major_version = deserializer.try_get_varint_u32_le()?;
        let minor_version = deserializer.try_get_varint_u32_le()?;
        let value = deserializer.finish()?;

        Ok(Self {
            major_version,
            minor_version,
            value,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
    }
}

impl Sealed for Connect2 {}

impl From<Connect2> for Message {
    fn from(msg: Connect2) -> Self {
        Self::Connect2(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::{Connect2, ConnectData};
    use crate::SerializedValue;

    #[test]
    fn connect2() {
        let serialized = [13, 0, 0, 0, 46, 2, 0, 0, 0, 65, 0, 1, 2];
        let value = ConnectData::new();

        let msg = Connect2 {
            major_version: 1,
            minor_version: 2,
            value: SerializedValue::serialize(&value).unwrap(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::Connect2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
