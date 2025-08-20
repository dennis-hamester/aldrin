use super::{ir, resolve_ir, ArrayType, LexicalId, MapType, ResultType};
use crate::tags::{PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum BuiltInType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Uuid,
    ObjectId,
    ServiceId,
    Value,
    Option(TypeId),
    Box(TypeId),
    Vec(TypeId),
    Bytes,
    Map(MapType),
    Set(TypeId),
    Sender(TypeId),
    Receiver(TypeId),
    Lifetime,
    Unit,
    Result(ResultType),
    Array(ArrayType),
}

impl BuiltInType {
    pub fn from_ir(ty: ir::BuiltInTypeIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        match ty {
            ir::BuiltInTypeIr::Bool => Self::Bool,
            ir::BuiltInTypeIr::U8 => Self::U8,
            ir::BuiltInTypeIr::I8 => Self::I8,
            ir::BuiltInTypeIr::U16 => Self::U16,
            ir::BuiltInTypeIr::I16 => Self::I16,
            ir::BuiltInTypeIr::U32 => Self::U32,
            ir::BuiltInTypeIr::I32 => Self::I32,
            ir::BuiltInTypeIr::U64 => Self::U64,
            ir::BuiltInTypeIr::I64 => Self::I64,
            ir::BuiltInTypeIr::F32 => Self::F32,
            ir::BuiltInTypeIr::F64 => Self::F64,
            ir::BuiltInTypeIr::String => Self::String,
            ir::BuiltInTypeIr::Uuid => Self::Uuid,
            ir::BuiltInTypeIr::ObjectId => Self::ObjectId,
            ir::BuiltInTypeIr::ServiceId => Self::ServiceId,
            ir::BuiltInTypeIr::Value => Self::Value,
            ir::BuiltInTypeIr::Option(ty) => Self::Option(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Box(ty) => Self::Box(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Vec(ty) => Self::Vec(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Bytes => Self::Bytes,
            ir::BuiltInTypeIr::Map(ty) => Self::Map(MapType::from_ir(ty, references)),
            ir::BuiltInTypeIr::Set(ty) => Self::Set(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Sender(ty) => Self::Sender(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Receiver(ty) => Self::Receiver(resolve_ir(ty, references)),
            ir::BuiltInTypeIr::Lifetime => Self::Lifetime,
            ir::BuiltInTypeIr::Unit => Self::Unit,
            ir::BuiltInTypeIr::Result(ty) => Self::Result(ResultType::from_ir(ty, references)),
            ir::BuiltInTypeIr::Array(ty) => Self::Array(ArrayType::from_ir(ty, references)),
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum BuiltInTypeVariant {
    Bool = 0,
    U8 = 1,
    I8 = 2,
    U16 = 3,
    I16 = 4,
    U32 = 5,
    I32 = 6,
    U64 = 7,
    I64 = 8,
    F32 = 9,
    F64 = 10,
    String = 11,
    Uuid = 12,
    ObjectId = 13,
    ServiceId = 14,
    Value = 15,
    Option = 16,
    Box = 17,
    Vec = 18,
    Bytes = 19,
    Map = 20,
    Set = 21,
    Sender = 22,
    Receiver = 23,
    Lifetime = 24,
    Unit = 25,
    Result = 26,
    Array = 27,
}

impl Tag for BuiltInType {}

impl PrimaryTag for BuiltInType {
    type Tag = Self;
}

impl Serialize<Self> for BuiltInType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Bool => serializer.serialize_unit_enum(BuiltInTypeVariant::Bool),
            Self::U8 => serializer.serialize_unit_enum(BuiltInTypeVariant::U8),
            Self::I8 => serializer.serialize_unit_enum(BuiltInTypeVariant::I8),
            Self::U16 => serializer.serialize_unit_enum(BuiltInTypeVariant::U16),
            Self::I16 => serializer.serialize_unit_enum(BuiltInTypeVariant::I16),
            Self::U32 => serializer.serialize_unit_enum(BuiltInTypeVariant::U32),
            Self::I32 => serializer.serialize_unit_enum(BuiltInTypeVariant::I32),
            Self::U64 => serializer.serialize_unit_enum(BuiltInTypeVariant::U64),
            Self::I64 => serializer.serialize_unit_enum(BuiltInTypeVariant::I64),
            Self::F32 => serializer.serialize_unit_enum(BuiltInTypeVariant::F32),
            Self::F64 => serializer.serialize_unit_enum(BuiltInTypeVariant::F64),
            Self::String => serializer.serialize_unit_enum(BuiltInTypeVariant::String),
            Self::Uuid => serializer.serialize_unit_enum(BuiltInTypeVariant::Uuid),
            Self::ObjectId => serializer.serialize_unit_enum(BuiltInTypeVariant::ObjectId),
            Self::ServiceId => serializer.serialize_unit_enum(BuiltInTypeVariant::ServiceId),
            Self::Value => serializer.serialize_unit_enum(BuiltInTypeVariant::Value),

            Self::Option(t) => {
                serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Option, t)
            }

            Self::Box(t) => serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Box, t),
            Self::Vec(t) => serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Vec, t),
            Self::Bytes => serializer.serialize_unit_enum(BuiltInTypeVariant::Bytes),
            Self::Map(t) => serializer.serialize_enum::<MapType, _>(BuiltInTypeVariant::Map, t),
            Self::Set(t) => serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Set, t),

            Self::Sender(t) => {
                serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Sender, t)
            }

            Self::Receiver(t) => {
                serializer.serialize_enum::<TypeId, _>(BuiltInTypeVariant::Receiver, t)
            }

            Self::Lifetime => serializer.serialize_unit_enum(BuiltInTypeVariant::Lifetime),
            Self::Unit => serializer.serialize_unit_enum(BuiltInTypeVariant::Unit),

            Self::Result(t) => {
                serializer.serialize_enum::<ResultType, _>(BuiltInTypeVariant::Result, t)
            }

            Self::Array(t) => {
                serializer.serialize_enum::<ArrayType, _>(BuiltInTypeVariant::Array, t)
            }
        }
    }
}

impl Serialize<BuiltInType> for &BuiltInType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for BuiltInType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_id()? {
            BuiltInTypeVariant::Bool => deserializer.deserialize_unit().map(|()| Self::Bool),
            BuiltInTypeVariant::U8 => deserializer.deserialize_unit().map(|()| Self::U8),
            BuiltInTypeVariant::I8 => deserializer.deserialize_unit().map(|()| Self::I8),
            BuiltInTypeVariant::U16 => deserializer.deserialize_unit().map(|()| Self::U16),
            BuiltInTypeVariant::I16 => deserializer.deserialize_unit().map(|()| Self::I16),
            BuiltInTypeVariant::U32 => deserializer.deserialize_unit().map(|()| Self::U32),
            BuiltInTypeVariant::I32 => deserializer.deserialize_unit().map(|()| Self::I32),
            BuiltInTypeVariant::U64 => deserializer.deserialize_unit().map(|()| Self::U64),
            BuiltInTypeVariant::I64 => deserializer.deserialize_unit().map(|()| Self::I64),
            BuiltInTypeVariant::F32 => deserializer.deserialize_unit().map(|()| Self::F32),
            BuiltInTypeVariant::F64 => deserializer.deserialize_unit().map(|()| Self::F64),
            BuiltInTypeVariant::String => deserializer.deserialize_unit().map(|()| Self::String),
            BuiltInTypeVariant::Uuid => deserializer.deserialize_unit().map(|()| Self::Uuid),

            BuiltInTypeVariant::ObjectId => {
                deserializer.deserialize_unit().map(|()| Self::ObjectId)
            }

            BuiltInTypeVariant::ServiceId => {
                deserializer.deserialize_unit().map(|()| Self::ServiceId)
            }

            BuiltInTypeVariant::Value => deserializer.deserialize_unit().map(|()| Self::Value),

            BuiltInTypeVariant::Option => deserializer.deserialize::<TypeId, _>().map(Self::Option),

            BuiltInTypeVariant::Box => deserializer.deserialize::<TypeId, _>().map(Self::Box),
            BuiltInTypeVariant::Vec => deserializer.deserialize::<TypeId, _>().map(Self::Vec),
            BuiltInTypeVariant::Bytes => deserializer.deserialize_unit().map(|()| Self::Bytes),
            BuiltInTypeVariant::Map => deserializer.deserialize::<MapType, _>().map(Self::Map),
            BuiltInTypeVariant::Set => deserializer.deserialize::<TypeId, _>().map(Self::Set),

            BuiltInTypeVariant::Sender => deserializer.deserialize::<TypeId, _>().map(Self::Sender),

            BuiltInTypeVariant::Receiver => {
                deserializer.deserialize::<TypeId, _>().map(Self::Receiver)
            }

            BuiltInTypeVariant::Lifetime => {
                deserializer.deserialize_unit().map(|()| Self::Lifetime)
            }

            BuiltInTypeVariant::Unit => deserializer.deserialize_unit().map(|()| Self::Unit),

            BuiltInTypeVariant::Result => deserializer
                .deserialize::<ResultType, _>()
                .map(Self::Result),

            BuiltInTypeVariant::Array => {
                deserializer.deserialize::<ArrayType, _>().map(Self::Array)
            }
        }
    }
}
