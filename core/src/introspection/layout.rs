use super::{ir, BuiltInType, Enum, LexicalId, Newtype, Service, Struct};
use crate::tags::{PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layout {
    BuiltIn(BuiltInType),
    Struct(Struct),
    Enum(Enum),
    Service(Service),
    Newtype(Newtype),
}

impl Layout {
    pub fn from_ir(layout: ir::LayoutIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        match layout {
            ir::LayoutIr::BuiltIn(ty) => Self::BuiltIn(BuiltInType::from_ir(ty, references)),
            ir::LayoutIr::Struct(ty) => Self::Struct(Struct::from_ir(ty, references)),
            ir::LayoutIr::Enum(ty) => Self::Enum(Enum::from_ir(ty, references)),
            ir::LayoutIr::Service(ty) => Self::Service(Service::from_ir(ty, references)),
            ir::LayoutIr::Newtype(ty) => Self::Newtype(Newtype::from_ir(ty, references)),
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

    pub fn as_newtype(&self) -> Option<&Newtype> {
        match self {
            Self::Newtype(ty) => Some(ty),
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

impl From<Newtype> for Layout {
    fn from(ty: Newtype) -> Self {
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

impl Tag for Layout {}

impl PrimaryTag for Layout {
    type Tag = Self;
}

impl Serialize<Self> for Layout {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Layout> for &Layout {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Layout::BuiltIn(ty) => {
                serializer.serialize_enum::<BuiltInType, _>(LayoutVariant::BuiltIn, ty)
            }

            Layout::Struct(ty) => serializer.serialize_enum::<Struct, _>(LayoutVariant::Struct, ty),
            Layout::Enum(ty) => serializer.serialize_enum::<Enum, _>(LayoutVariant::Enum, ty),

            Layout::Service(ty) => {
                serializer.serialize_enum::<Service, _>(LayoutVariant::Service, ty)
            }

            Layout::Newtype(ty) => {
                serializer.serialize_enum::<Newtype, _>(LayoutVariant::Newtype, ty)
            }
        }
    }
}

impl Deserialize<Self> for Layout {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_id()? {
            LayoutVariant::BuiltIn => deserializer
                .deserialize::<BuiltInType, _>()
                .map(Self::BuiltIn),

            LayoutVariant::Struct => deserializer.deserialize::<Struct, _>().map(Self::Struct),
            LayoutVariant::Enum => deserializer.deserialize::<Enum, _>().map(Self::Enum),
            LayoutVariant::Service => deserializer.deserialize::<Service, _>().map(Self::Service),
            LayoutVariant::Newtype => deserializer.deserialize::<Newtype, _>().map(Self::Newtype),
        }
    }
}
