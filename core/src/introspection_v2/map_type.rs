use super::{KeyType, LexicalId};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MapType {
    key: KeyType,
    value: LexicalId,
}

impl MapType {
    pub fn new(key: KeyType, value: LexicalId) -> Self {
        Self { key, value }
    }

    pub fn key(self) -> KeyType {
        self.key
    }

    pub fn value(self) -> LexicalId {
        self.value
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MapTypeField {
    Key = 0,
    Value = 1,
}

impl Serialize for MapType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(MapTypeField::Key, &self.key)?;
        serializer.serialize_field(MapTypeField::Value, &self.value)?;

        serializer.finish()
    }
}

impl Deserialize for MapType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let key = deserializer.deserialize_specific_field(MapTypeField::Key)?;
        let value = deserializer.deserialize_specific_field(MapTypeField::Value)?;

        deserializer.finish(Self { key, value })
    }
}
