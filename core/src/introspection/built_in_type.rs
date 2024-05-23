use super::{KeyType, MapType, ResultType, TypeRef};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Option(Box<TypeRef>),
    Box(Box<TypeRef>),
    Vec(Box<TypeRef>),
    Bytes,
    Map(Box<MapType>),
    Set(KeyType),
    Sender(Box<TypeRef>),
    Receiver(Box<TypeRef>),
    Lifetime,
    Unit,
    Result(Box<ResultType>),
}

impl From<BuiltInType> for TypeRef {
    fn from(t: BuiltInType) -> Self {
        Self::BuiltIn(t)
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
}

impl Serialize for BuiltInType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Bool => serializer.serialize_enum(BuiltInTypeVariant::Bool, &()),
            Self::U8 => serializer.serialize_enum(BuiltInTypeVariant::U8, &()),
            Self::I8 => serializer.serialize_enum(BuiltInTypeVariant::I8, &()),
            Self::U16 => serializer.serialize_enum(BuiltInTypeVariant::U16, &()),
            Self::I16 => serializer.serialize_enum(BuiltInTypeVariant::I16, &()),
            Self::U32 => serializer.serialize_enum(BuiltInTypeVariant::U32, &()),
            Self::I32 => serializer.serialize_enum(BuiltInTypeVariant::I32, &()),
            Self::U64 => serializer.serialize_enum(BuiltInTypeVariant::U64, &()),
            Self::I64 => serializer.serialize_enum(BuiltInTypeVariant::I64, &()),
            Self::F32 => serializer.serialize_enum(BuiltInTypeVariant::F32, &()),
            Self::F64 => serializer.serialize_enum(BuiltInTypeVariant::F64, &()),
            Self::String => serializer.serialize_enum(BuiltInTypeVariant::String, &()),
            Self::Uuid => serializer.serialize_enum(BuiltInTypeVariant::Uuid, &()),
            Self::ObjectId => serializer.serialize_enum(BuiltInTypeVariant::ObjectId, &()),
            Self::ServiceId => serializer.serialize_enum(BuiltInTypeVariant::ServiceId, &()),
            Self::Value => serializer.serialize_enum(BuiltInTypeVariant::Value, &()),
            Self::Option(t) => serializer.serialize_enum(BuiltInTypeVariant::Option, t),
            Self::Box(t) => serializer.serialize_enum(BuiltInTypeVariant::Box, t),
            Self::Vec(t) => serializer.serialize_enum(BuiltInTypeVariant::Vec, t),
            Self::Bytes => serializer.serialize_enum(BuiltInTypeVariant::Bytes, &()),
            Self::Map(t) => serializer.serialize_enum(BuiltInTypeVariant::Map, t),
            Self::Set(t) => serializer.serialize_enum(BuiltInTypeVariant::Set, t),
            Self::Sender(t) => serializer.serialize_enum(BuiltInTypeVariant::Sender, t),
            Self::Receiver(t) => serializer.serialize_enum(BuiltInTypeVariant::Receiver, t),
            Self::Lifetime => serializer.serialize_enum(BuiltInTypeVariant::Lifetime, &()),
            Self::Unit => serializer.serialize_enum(BuiltInTypeVariant::Unit, &()),
            Self::Result(t) => serializer.serialize_enum(BuiltInTypeVariant::Result, t),
        }
    }
}

impl Deserialize for BuiltInType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_variant()? {
            BuiltInTypeVariant::Bool => deserializer.deserialize().map(|()| Self::Bool),
            BuiltInTypeVariant::U8 => deserializer.deserialize().map(|()| Self::U8),
            BuiltInTypeVariant::I8 => deserializer.deserialize().map(|()| Self::I8),
            BuiltInTypeVariant::U16 => deserializer.deserialize().map(|()| Self::U16),
            BuiltInTypeVariant::I16 => deserializer.deserialize().map(|()| Self::I16),
            BuiltInTypeVariant::U32 => deserializer.deserialize().map(|()| Self::U32),
            BuiltInTypeVariant::I32 => deserializer.deserialize().map(|()| Self::I32),
            BuiltInTypeVariant::U64 => deserializer.deserialize().map(|()| Self::U64),
            BuiltInTypeVariant::I64 => deserializer.deserialize().map(|()| Self::I64),
            BuiltInTypeVariant::F32 => deserializer.deserialize().map(|()| Self::F32),
            BuiltInTypeVariant::F64 => deserializer.deserialize().map(|()| Self::F64),
            BuiltInTypeVariant::String => deserializer.deserialize().map(|()| Self::String),
            BuiltInTypeVariant::Uuid => deserializer.deserialize().map(|()| Self::Uuid),
            BuiltInTypeVariant::ObjectId => deserializer.deserialize().map(|()| Self::ObjectId),
            BuiltInTypeVariant::ServiceId => deserializer.deserialize().map(|()| Self::ServiceId),
            BuiltInTypeVariant::Value => deserializer.deserialize().map(|()| Self::Value),
            BuiltInTypeVariant::Option => deserializer.deserialize().map(Self::Option),
            BuiltInTypeVariant::Box => deserializer.deserialize().map(Self::Box),
            BuiltInTypeVariant::Vec => deserializer.deserialize().map(Self::Vec),
            BuiltInTypeVariant::Bytes => deserializer.deserialize().map(|()| Self::Bytes),
            BuiltInTypeVariant::Map => deserializer.deserialize().map(Self::Map),
            BuiltInTypeVariant::Set => deserializer.deserialize().map(Self::Set),
            BuiltInTypeVariant::Sender => deserializer.deserialize().map(Self::Sender),
            BuiltInTypeVariant::Receiver => deserializer.deserialize().map(Self::Receiver),
            BuiltInTypeVariant::Lifetime => deserializer.deserialize().map(|()| Self::Lifetime),
            BuiltInTypeVariant::Unit => deserializer.deserialize().map(|()| Self::Unit),
            BuiltInTypeVariant::Result => deserializer.deserialize().map(Self::Result),
        }
    }
}

impl fmt::Display for BuiltInType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::U8 => write!(f, "u8"),
            Self::I8 => write!(f, "i8"),
            Self::U16 => write!(f, "u16"),
            Self::I16 => write!(f, "i16"),
            Self::U32 => write!(f, "u32"),
            Self::I32 => write!(f, "i32"),
            Self::U64 => write!(f, "u64"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
            Self::Uuid => write!(f, "uuid"),
            Self::ObjectId => write!(f, "object_id"),
            Self::ServiceId => write!(f, "service_id"),
            Self::Value => write!(f, "value"),
            Self::Option(inner) => write!(f, "option<{inner}>"),
            Self::Box(inner) => write!(f, "box<{inner}>"),
            Self::Vec(inner) => write!(f, "vec<{inner}>"),
            Self::Bytes => write!(f, "bytes"),
            Self::Map(inner) => inner.fmt(f),
            Self::Set(inner) => write!(f, "set<{inner}>"),
            Self::Sender(inner) => write!(f, "sender<{inner}>"),
            Self::Receiver(inner) => write!(f, "receiver<{inner}>"),
            Self::Lifetime => write!(f, "lifetime"),
            Self::Unit => write!(f, "unit"),
            Self::Result(inner) => inner.fmt(f),
        }
    }
}
