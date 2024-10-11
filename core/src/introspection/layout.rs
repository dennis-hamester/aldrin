use super::{Enum, Service, Struct};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Layout {
    Service(Service),
    Struct(Struct),
    Enum(Enum),
}

impl Layout {
    pub fn namespace(&self) -> Uuid {
        match self {
            Self::Service(_) => Service::NAMESPACE,
            Self::Struct(_) => Struct::NAMESPACE,
            Self::Enum(_) => Enum::NAMESPACE,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Service(s) => s.name(),
            Self::Struct(s) => s.name(),
            Self::Enum(e) => e.name(),
        }
    }

    pub fn as_service(&self) -> Option<&Service> {
        match self {
            Self::Service(service) => Some(service),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Self::Struct(struct_) => Some(struct_),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&Enum> {
        match self {
            Self::Enum(enum_) => Some(enum_),
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

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    Service = 0,
    Struct = 1,
    Enum = 2,
}

impl Serialize for Layout {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Service(l) => serializer.serialize_enum(LayoutVariant::Service, l),
            Self::Struct(l) => serializer.serialize_enum(LayoutVariant::Struct, l),
            Self::Enum(l) => serializer.serialize_enum(LayoutVariant::Enum, l),
        }
    }
}

impl Deserialize for Layout {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            LayoutVariant::Service => deserializer.deserialize().map(Self::Service),
            LayoutVariant::Struct => deserializer.deserialize().map(Self::Struct),
            LayoutVariant::Enum => deserializer.deserialize().map(Self::Enum),
        }
    }
}
