use super::BuiltInType;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layout {
    BuiltIn(BuiltInType),
}

impl Layout {
    pub fn namespace(&self) -> Uuid {
        match self {
            Self::BuiltIn(_) => BuiltInType::NAMESPACE,
        }
    }

    pub fn as_built_in(&self) -> Option<BuiltInType> {
        match self {
            Self::BuiltIn(ty) => Some(*ty),
        }
    }
}

impl From<BuiltInType> for Layout {
    fn from(ty: BuiltInType) -> Self {
        Self::BuiltIn(ty)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    BuiltIn = 0,
}

impl Serialize for Layout {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(ty) => serializer.serialize_enum(LayoutVariant::BuiltIn, ty),
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
