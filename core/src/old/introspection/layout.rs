use super::{BuiltInType, Enum, LexicalId, Service, Struct};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layout {
    BuiltIn(BuiltInType),
    Struct(Struct),
    Enum(Enum),
    Service(Service),
}

impl Layout {
    pub fn namespace(&self) -> Uuid {
        match self {
            Self::BuiltIn(_) => BuiltInType::NAMESPACE,
            Self::Struct(_) => Struct::NAMESPACE,
            Self::Enum(_) => Enum::NAMESPACE,
            Self::Service(_) => Service::NAMESPACE,
        }
    }

    pub fn lexical_id(&self) -> LexicalId {
        match self {
            Self::BuiltIn(ty) => ty.lexical_id(),
            Self::Struct(ty) => ty.lexical_id(),
            Self::Enum(ty) => ty.lexical_id(),
            Self::Service(ty) => ty.lexical_id(),
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

    pub fn as_enum(&self) -> Option<&Enum> {
        match self {
            Self::Enum(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn as_service(&self) -> Option<&Service> {
        match self {
            Self::Service(ty) => Some(ty),
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

impl From<Enum> for Layout {
    fn from(ty: Enum) -> Self {
        Self::Enum(ty)
    }
}

impl From<Service> for Layout {
    fn from(ty: Service) -> Self {
        Self::Service(ty)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    BuiltIn = 0,
    Struct = 1,
    Enum = 2,
    Service = 3,
}

impl Serialize for Layout {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(ty) => serializer.serialize_enum(LayoutVariant::BuiltIn, ty),
            Self::Struct(ty) => serializer.serialize_enum(LayoutVariant::Struct, ty),
            Self::Enum(ty) => serializer.serialize_enum(LayoutVariant::Enum, ty),
            Self::Service(ty) => serializer.serialize_enum(LayoutVariant::Service, ty),
        }
    }
}

impl Deserialize for Layout {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            LayoutVariant::BuiltIn => deserializer.deserialize().map(Self::BuiltIn),
            LayoutVariant::Struct => deserializer.deserialize().map(Self::Struct),
            LayoutVariant::Enum => deserializer.deserialize().map(Self::Enum),
            LayoutVariant::Service => deserializer.deserialize().map(Self::Service),
        }
    }
}
