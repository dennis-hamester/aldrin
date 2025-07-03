use super::{ArrayType, LexicalId, MapType, ResultType};
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::{uuid, Uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    Option(LexicalId),
    Box(LexicalId),
    Vec(LexicalId),
    Bytes,
    Map(MapType),
    Set(LexicalId),
    Sender(LexicalId),
    Receiver(LexicalId),
    Lifetime,
    Unit,
    Result(ResultType),
    Array(ArrayType),
}

impl BuiltInType {
    pub const NAMESPACE: Uuid = uuid!("43852cf9-014c-44f1-86d7-0b1b753eeb02");

    pub fn lexical_id(self) -> LexicalId {
        match self {
            Self::Bool => LexicalId::BOOL,
            Self::U8 => LexicalId::U8,
            Self::I8 => LexicalId::I8,
            Self::U16 => LexicalId::U16,
            Self::I16 => LexicalId::I16,
            Self::U32 => LexicalId::U32,
            Self::I32 => LexicalId::I32,
            Self::U64 => LexicalId::U64,
            Self::I64 => LexicalId::I64,
            Self::F32 => LexicalId::F32,
            Self::F64 => LexicalId::F64,
            Self::String => LexicalId::STRING,
            Self::Uuid => LexicalId::UUID,
            Self::ObjectId => LexicalId::OBJECT_ID,
            Self::ServiceId => LexicalId::SERVICE_ID,
            Self::Value => LexicalId::VALUE,
            Self::Option(ty) => LexicalId::option(ty),
            Self::Box(ty) => LexicalId::box_ty(ty),
            Self::Vec(ty) => LexicalId::vec(ty),
            Self::Bytes => LexicalId::BYTES,
            Self::Map(ty) => LexicalId::map(ty.key(), ty.value()),
            Self::Set(ty) => LexicalId::set(ty),
            Self::Sender(ty) => LexicalId::sender(ty),
            Self::Receiver(ty) => LexicalId::receiver(ty),
            Self::Lifetime => LexicalId::LIFETIME,
            Self::Unit => LexicalId::UNIT,
            Self::Result(ty) => LexicalId::result(ty.ok(), ty.err()),
            Self::Array(arr) => LexicalId::array(arr.elem_type(), arr.len()),
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
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Option, t)
            }

            Self::Box(t) => serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Box, t),
            Self::Vec(t) => serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Vec, t),
            Self::Bytes => serializer.serialize_unit_enum(BuiltInTypeVariant::Bytes),
            Self::Map(t) => serializer.serialize_enum::<MapType, _>(BuiltInTypeVariant::Map, t),
            Self::Set(t) => serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Set, t),

            Self::Sender(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Sender, t)
            }

            Self::Receiver(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Receiver, t)
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

            BuiltInTypeVariant::Option => {
                deserializer.deserialize::<LexicalId, _>().map(Self::Option)
            }

            BuiltInTypeVariant::Box => deserializer.deserialize::<LexicalId, _>().map(Self::Box),
            BuiltInTypeVariant::Vec => deserializer.deserialize::<LexicalId, _>().map(Self::Vec),
            BuiltInTypeVariant::Bytes => deserializer.deserialize_unit().map(|()| Self::Bytes),
            BuiltInTypeVariant::Map => deserializer.deserialize::<MapType, _>().map(Self::Map),
            BuiltInTypeVariant::Set => deserializer.deserialize::<LexicalId, _>().map(Self::Set),

            BuiltInTypeVariant::Sender => {
                deserializer.deserialize::<LexicalId, _>().map(Self::Sender)
            }

            BuiltInTypeVariant::Receiver => deserializer
                .deserialize::<LexicalId, _>()
                .map(Self::Receiver),

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
