use super::{BuiltInType, CustomType};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    BuiltIn(BuiltInType),
    Custom(CustomType),
}

impl TypeRef {
    pub fn custom(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Custom(CustomType::new(schema, name))
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum TypeRefVariant {
    BuiltIn = 0,
    Custom = 1,
}

impl Serialize for TypeRef {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(t) => serializer.serialize_enum(TypeRefVariant::BuiltIn, t),
            Self::Custom(t) => serializer.serialize_enum(TypeRefVariant::Custom, t),
        }
    }
}

impl Deserialize for TypeRef {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            TypeRefVariant::BuiltIn => deserializer.deserialize().map(Self::BuiltIn),
            TypeRefVariant::Custom => deserializer.deserialize().map(Self::Custom),
        }
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BuiltIn(inner) => inner.fmt(f),
            Self::Custom(inner) => inner.fmt(f),
        }
    }
}
