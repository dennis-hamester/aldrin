use super::{BuiltInType, Struct};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layout {
    BuiltIn(BuiltInType),
    Struct(Struct),
}

impl Layout {
    pub fn namespace(&self) -> Uuid {
        match self {
            Self::BuiltIn(_) => BuiltInType::NAMESPACE,
            Self::Struct(_) => Struct::NAMESPACE,
        }
    }

    pub fn as_built_in(&self) -> Option<BuiltInType> {
        match self {
            Self::BuiltIn(ty) => Some(*ty),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Self::Struct(ty) => Some(ty),
            _ => None,
        }
    }
}

impl From<BuiltInType> for Layout {
    fn from(ty: BuiltInType) -> Self {
        Self::BuiltIn(ty)
    }
}

impl From<Struct> for Layout {
    fn from(ty: Struct) -> Self {
        Self::Struct(ty)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    BuiltIn = 0,
    Struct = 1,
}

impl Serialize for Layout {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(ty) => serializer.serialize_enum(LayoutVariant::BuiltIn, ty),
            Self::Struct(ty) => serializer.serialize_enum(LayoutVariant::Struct, ty),
        }
    }
}

impl Deserialize for Layout {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            LayoutVariant::BuiltIn => deserializer.deserialize().map(Self::BuiltIn),
            LayoutVariant::Struct => deserializer.deserialize().map(Self::Struct),
        }
    }
}
