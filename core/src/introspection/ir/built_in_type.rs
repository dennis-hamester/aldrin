use super::{ArrayTypeIr, LexicalId, MapTypeIr, ResultTypeIr};
use crate::tags::{PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::{uuid, Uuid};

#[derive(Debug, Copy, Clone)]
pub enum BuiltInTypeIr {
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
    Map(MapTypeIr),
    Set(LexicalId),
    Sender(LexicalId),
    Receiver(LexicalId),
    Lifetime,
    Unit,
    Result(ResultTypeIr),
    Array(ArrayTypeIr),
}

impl BuiltInTypeIr {
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

impl Tag for BuiltInTypeIr {}

impl PrimaryTag for BuiltInTypeIr {
    type Tag = Self;
}

impl Serialize<BuiltInTypeIr> for &BuiltInTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            BuiltInTypeIr::Bool => serializer.serialize_unit_enum(BuiltInTypeVariant::Bool),
            BuiltInTypeIr::U8 => serializer.serialize_unit_enum(BuiltInTypeVariant::U8),
            BuiltInTypeIr::I8 => serializer.serialize_unit_enum(BuiltInTypeVariant::I8),
            BuiltInTypeIr::U16 => serializer.serialize_unit_enum(BuiltInTypeVariant::U16),
            BuiltInTypeIr::I16 => serializer.serialize_unit_enum(BuiltInTypeVariant::I16),
            BuiltInTypeIr::U32 => serializer.serialize_unit_enum(BuiltInTypeVariant::U32),
            BuiltInTypeIr::I32 => serializer.serialize_unit_enum(BuiltInTypeVariant::I32),
            BuiltInTypeIr::U64 => serializer.serialize_unit_enum(BuiltInTypeVariant::U64),
            BuiltInTypeIr::I64 => serializer.serialize_unit_enum(BuiltInTypeVariant::I64),
            BuiltInTypeIr::F32 => serializer.serialize_unit_enum(BuiltInTypeVariant::F32),
            BuiltInTypeIr::F64 => serializer.serialize_unit_enum(BuiltInTypeVariant::F64),
            BuiltInTypeIr::String => serializer.serialize_unit_enum(BuiltInTypeVariant::String),
            BuiltInTypeIr::Uuid => serializer.serialize_unit_enum(BuiltInTypeVariant::Uuid),
            BuiltInTypeIr::ObjectId => serializer.serialize_unit_enum(BuiltInTypeVariant::ObjectId),

            BuiltInTypeIr::ServiceId => {
                serializer.serialize_unit_enum(BuiltInTypeVariant::ServiceId)
            }

            BuiltInTypeIr::Value => serializer.serialize_unit_enum(BuiltInTypeVariant::Value),

            BuiltInTypeIr::Option(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Option, t)
            }

            BuiltInTypeIr::Box(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Box, t)
            }

            BuiltInTypeIr::Vec(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Vec, t)
            }

            BuiltInTypeIr::Bytes => serializer.serialize_unit_enum(BuiltInTypeVariant::Bytes),

            BuiltInTypeIr::Map(t) => {
                serializer.serialize_enum::<MapTypeIr, _>(BuiltInTypeVariant::Map, t)
            }

            BuiltInTypeIr::Set(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Set, t)
            }

            BuiltInTypeIr::Sender(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Sender, t)
            }

            BuiltInTypeIr::Receiver(t) => {
                serializer.serialize_enum::<LexicalId, _>(BuiltInTypeVariant::Receiver, t)
            }

            BuiltInTypeIr::Lifetime => serializer.serialize_unit_enum(BuiltInTypeVariant::Lifetime),
            BuiltInTypeIr::Unit => serializer.serialize_unit_enum(BuiltInTypeVariant::Unit),

            BuiltInTypeIr::Result(t) => {
                serializer.serialize_enum::<ResultTypeIr, _>(BuiltInTypeVariant::Result, t)
            }

            BuiltInTypeIr::Array(t) => {
                serializer.serialize_enum::<ArrayTypeIr, _>(BuiltInTypeVariant::Array, t)
            }
        }
    }
}
