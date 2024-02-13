use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::{DeserializeError, SerializeError};
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct ConnectData {
    pub user: Option<SerializedValue>,
}

impl ConnectData {
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

impl Serialize for ConnectData {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(1)?;
        serializer.serialize_field(0, &self.user)?;
        serializer.finish()
    }
}

impl Deserialize for ConnectData {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut user = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.id() {
                0 => user = deserializer.deserialize()?,
                _ => deserializer.skip()?,
            }
        }

        Ok(Self { user })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct Connect2 {
    pub major_version: u32,
    pub minor_version: u32,
    pub value: SerializedValue,
}

impl Connect2 {
    pub fn with_serialize_data(
        major_version: u32,
        minor_version: u32,
        data: &ConnectData,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(data)?;

        Ok(Self {
            major_version,
            minor_version,
            value,
        })
    }

    pub fn deserialize_connect_data(&self) -> Result<ConnectData, DeserializeError> {
        self.value.deserialize()
    }
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

    #[test]
    fn connect() {
        let serialized = [15, 0, 0, 0, 46, 4, 0, 0, 0, 39, 1, 0, 0, 1, 2];
        let value = ConnectData::new();

        let msg = Connect2::with_serialize_data(1, 2, &value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::Connect2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
