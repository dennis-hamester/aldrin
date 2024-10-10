use super::BuiltInType;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub enum Layout {
    BuiltIn(BuiltInType),
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    BuiltIn = 0,
}

impl Serialize for Layout {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(l) => serializer.serialize_enum(LayoutVariant::BuiltIn, l),
        }
    }
}

impl Deserialize for Layout {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            LayoutVariant::BuiltIn => deserializer.deserialize().map(Self::BuiltIn),
        }
    }
}
