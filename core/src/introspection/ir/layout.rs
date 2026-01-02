use super::{BuiltInTypeIr, EnumIr, LexicalId, NewtypeIr, ServiceIr, StructIr};
use crate::tags::{PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum LayoutIr {
    BuiltIn(BuiltInTypeIr),
    Struct(StructIr),
    Enum(EnumIr),
    Service(ServiceIr),
    Newtype(NewtypeIr),
}

impl LayoutIr {
    pub fn namespace(&self) -> Uuid {
        match self {
            Self::BuiltIn(_) => BuiltInTypeIr::NAMESPACE,
            Self::Struct(_) => StructIr::NAMESPACE,
            Self::Enum(_) => EnumIr::NAMESPACE,
            Self::Service(_) => ServiceIr::NAMESPACE,
            Self::Newtype(_) => NewtypeIr::NAMESPACE,
        }
    }

    pub fn lexical_id(&self) -> LexicalId {
        match self {
            Self::BuiltIn(ty) => ty.lexical_id(),
            Self::Struct(ty) => ty.lexical_id(),
            Self::Enum(ty) => ty.lexical_id(),
            Self::Service(ty) => ty.lexical_id(),
            Self::Newtype(ty) => ty.lexical_id(),
        }
    }

    pub fn as_built_in(&self) -> Option<BuiltInTypeIr> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::BuiltIn(ty) => Some(*ty),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&StructIr> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Struct(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumIr> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Enum(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn as_service(&self) -> Option<&ServiceIr> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Service(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn as_newtype(&self) -> Option<&NewtypeIr> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Newtype(ty) => Some(ty),
            _ => None,
        }
    }
}

impl From<BuiltInTypeIr> for LayoutIr {
    fn from(ty: BuiltInTypeIr) -> Self {
        Self::BuiltIn(ty)
    }
}

impl From<StructIr> for LayoutIr {
    fn from(ty: StructIr) -> Self {
        Self::Struct(ty)
    }
}

impl From<EnumIr> for LayoutIr {
    fn from(ty: EnumIr) -> Self {
        Self::Enum(ty)
    }
}

impl From<ServiceIr> for LayoutIr {
    fn from(ty: ServiceIr) -> Self {
        Self::Service(ty)
    }
}

impl From<NewtypeIr> for LayoutIr {
    fn from(ty: NewtypeIr) -> Self {
        Self::Newtype(ty)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum LayoutVariant {
    BuiltIn = 0,
    Struct = 1,
    Enum = 2,
    Service = 3,
    Newtype = 4,
}

impl Tag for LayoutIr {}

impl PrimaryTag for LayoutIr {
    type Tag = Self;
}

impl Serialize<LayoutIr> for &LayoutIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            LayoutIr::BuiltIn(ty) => {
                serializer.serialize_enum::<BuiltInTypeIr>(LayoutVariant::BuiltIn, ty)
            }

            LayoutIr::Struct(ty) => {
                serializer.serialize_enum::<StructIr>(LayoutVariant::Struct, ty)
            }

            LayoutIr::Enum(ty) => serializer.serialize_enum::<EnumIr>(LayoutVariant::Enum, ty),

            LayoutIr::Service(ty) => {
                serializer.serialize_enum::<ServiceIr>(LayoutVariant::Service, ty)
            }

            LayoutIr::Newtype(ty) => {
                serializer.serialize_enum::<NewtypeIr>(LayoutVariant::Newtype, ty)
            }
        }
    }
}
