use super::{BuiltInType, KeyType, TypeRef};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapType {
    key: KeyType,
    value: TypeRef,
}

impl MapType {
    pub fn new(key: KeyType, value: impl Into<TypeRef>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }

    pub fn key(&self) -> KeyType {
        self.key
    }

    pub fn value(&self) -> &TypeRef {
        &self.value
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

impl From<MapType> for BuiltInType {
    fn from(t: MapType) -> Self {
        BuiltInType::Map(Box::new(t))
    }
}

impl From<MapType> for TypeRef {
    fn from(t: MapType) -> Self {
        Self::BuiltIn(t.into())
    }
}

impl fmt::Display for MapType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "map<{} -> {}>", self.key, self.value)
    }
}
