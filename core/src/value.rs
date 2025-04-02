#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Bytes, ChannelCookie, Deserialize, DeserializeError, Deserializer, ObjectId, Serialize,
    SerializeError, Serializer, ServiceId, ValueKind,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Value {
    None,
    Some(Box<Self>),
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Uuid(Uuid),
    ObjectId(ObjectId),
    ServiceId(ServiceId),
    Vec(Vec<Self>),
    Bytes(Bytes),
    U8Map(HashMap<u8, Self>),
    I8Map(HashMap<i8, Self>),
    U16Map(HashMap<u16, Self>),
    I16Map(HashMap<i16, Self>),
    U32Map(HashMap<u32, Self>),
    I32Map(HashMap<i32, Self>),
    U64Map(HashMap<u64, Self>),
    I64Map(HashMap<i64, Self>),
    StringMap(HashMap<String, Self>),
    UuidMap(HashMap<Uuid, Self>),
    U8Set(HashSet<u8>),
    I8Set(HashSet<i8>),
    U16Set(HashSet<u16>),
    I16Set(HashSet<i16>),
    U32Set(HashSet<u32>),
    I32Set(HashSet<i32>),
    U64Set(HashSet<u64>),
    I64Set(HashSet<i64>),
    StringSet(HashSet<String>),
    UuidSet(HashSet<Uuid>),
    Struct(Struct),
    Enum(Box<Enum>),
    Sender(ChannelCookie),
    Receiver(ChannelCookie),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl PrimaryTag for Value {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<tags::Value> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Some(value) => serializer.serialize_some(value),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::U8(value) => serializer.serialize_u8(*value),
            Value::I8(value) => serializer.serialize_i8(*value),
            Value::U16(value) => serializer.serialize_u16(*value),
            Value::I16(value) => serializer.serialize_i16(*value),
            Value::U32(value) => serializer.serialize_u32(*value),
            Value::I32(value) => serializer.serialize_i32(*value),
            Value::U64(value) => serializer.serialize_u64(*value),
            Value::I64(value) => serializer.serialize_i64(*value),
            Value::F32(value) => serializer.serialize_f32(*value),
            Value::F64(value) => serializer.serialize_f64(*value),
            Value::String(value) => serializer.serialize_string(value),
            Value::Uuid(value) => serializer.serialize_uuid(*value),
            Value::ObjectId(value) => serializer.serialize_object_id(*value),
            Value::ServiceId(value) => serializer.serialize_service_id(*value),
            Value::Vec(value) => serializer.serialize_vec2_iter(value),
            Value::Bytes(value) => serializer.serialize_byte_slice2(value),
            Value::U8Map(value) => serializer.serialize_map2_iter::<tags::U8, _, _, _, _>(value),
            Value::I8Map(value) => serializer.serialize_map2_iter::<tags::I8, _, _, _, _>(value),
            Value::U16Map(value) => serializer.serialize_map2_iter::<tags::U16, _, _, _, _>(value),
            Value::I16Map(value) => serializer.serialize_map2_iter::<tags::I16, _, _, _, _>(value),
            Value::U32Map(value) => serializer.serialize_map2_iter::<tags::U32, _, _, _, _>(value),
            Value::I32Map(value) => serializer.serialize_map2_iter::<tags::I32, _, _, _, _>(value),
            Value::U64Map(value) => serializer.serialize_map2_iter::<tags::U64, _, _, _, _>(value),
            Value::I64Map(value) => serializer.serialize_map2_iter::<tags::I64, _, _, _, _>(value),

            Value::StringMap(value) => {
                serializer.serialize_map2_iter::<tags::String, _, _, _, _>(value)
            }

            Value::UuidMap(value) => {
                serializer.serialize_map2_iter::<tags::Uuid, _, _, _, _>(value)
            }

            Value::U8Set(value) => serializer.serialize_set_iter::<tags::U8, _>(value),
            Value::I8Set(value) => serializer.serialize_set_iter::<tags::I8, _>(value),
            Value::U16Set(value) => serializer.serialize_set_iter::<tags::U16, _>(value),
            Value::I16Set(value) => serializer.serialize_set_iter::<tags::I16, _>(value),
            Value::U32Set(value) => serializer.serialize_set_iter::<tags::U32, _>(value),
            Value::I32Set(value) => serializer.serialize_set_iter::<tags::I32, _>(value),
            Value::U64Set(value) => serializer.serialize_set_iter::<tags::U64, _>(value),
            Value::I64Set(value) => serializer.serialize_set_iter::<tags::I64, _>(value),
            Value::StringSet(value) => serializer.serialize_set_iter::<tags::String, _>(value),
            Value::UuidSet(value) => serializer.serialize_set_iter::<tags::Uuid, _>(value),
            Value::Struct(value) => serializer.serialize::<_, _>(value),
            Value::Enum(value) => serializer.serialize::<_, _>(value),
            Value::Sender(value) => serializer.serialize_sender(*value),
            Value::Receiver(value) => serializer.serialize_receiver(*value),
        }
    }
}

impl Deserialize<tags::Value> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|()| Self::None),
            ValueKind::Some => deserializer.deserialize_some().map(Self::Some),
            ValueKind::Bool => deserializer.deserialize_bool().map(Self::Bool),
            ValueKind::U8 => deserializer.deserialize_u8().map(Self::U8),
            ValueKind::I8 => deserializer.deserialize_i8().map(Self::I8),
            ValueKind::U16 => deserializer.deserialize_u16().map(Self::U16),
            ValueKind::I16 => deserializer.deserialize_i16().map(Self::I16),
            ValueKind::U32 => deserializer.deserialize_u32().map(Self::U32),
            ValueKind::I32 => deserializer.deserialize_i32().map(Self::I32),
            ValueKind::U64 => deserializer.deserialize_u64().map(Self::U64),
            ValueKind::I64 => deserializer.deserialize_i64().map(Self::I64),
            ValueKind::F32 => deserializer.deserialize_f32().map(Self::F32),
            ValueKind::F64 => deserializer.deserialize_f64().map(Self::F64),
            ValueKind::String => deserializer.deserialize_string().map(Self::String),
            ValueKind::Uuid => deserializer.deserialize_uuid().map(Self::Uuid),
            ValueKind::ObjectId => deserializer.deserialize_object_id().map(Self::ObjectId),
            ValueKind::ServiceId => deserializer.deserialize_service_id().map(Self::ServiceId),
            ValueKind::Vec1 => deserializer.deserialize_vec1_extend_new().map(Self::Vec),

            ValueKind::Bytes1 => deserializer
                .deserialize_bytes1_extend_new()
                .map(Bytes)
                .map(Self::Bytes),

            ValueKind::U8Map1 => deserializer
                .deserialize_map1_extend_new::<tags::U8, _, _, _, _>()
                .map(Self::U8Map),

            ValueKind::I8Map1 => deserializer
                .deserialize_map1_extend_new::<tags::I8, _, _, _, _>()
                .map(Self::I8Map),

            ValueKind::U16Map1 => deserializer
                .deserialize_map1_extend_new::<tags::U16, _, _, _, _>()
                .map(Self::U16Map),

            ValueKind::I16Map1 => deserializer
                .deserialize_map1_extend_new::<tags::I16, _, _, _, _>()
                .map(Self::I16Map),

            ValueKind::U32Map1 => deserializer
                .deserialize_map1_extend_new::<tags::U32, _, _, _, _>()
                .map(Self::U32Map),

            ValueKind::I32Map1 => deserializer
                .deserialize_map1_extend_new::<tags::I32, _, _, _, _>()
                .map(Self::I32Map),

            ValueKind::U64Map1 => deserializer
                .deserialize_map1_extend_new::<tags::U64, _, _, _, _>()
                .map(Self::U64Map),

            ValueKind::I64Map1 => deserializer
                .deserialize_map1_extend_new::<tags::I64, _, _, _, _>()
                .map(Self::I64Map),

            ValueKind::StringMap1 => deserializer
                .deserialize_map1_extend_new::<tags::String, _, _, _, _>()
                .map(Self::StringMap),

            ValueKind::UuidMap1 => deserializer
                .deserialize_map1_extend_new::<tags::Uuid, _, _, _, _>()
                .map(Self::UuidMap),

            ValueKind::U8Set => deserializer
                .deserialize_set_extend_new::<tags::U8, u8, _>()
                .map(Self::U8Set),

            ValueKind::I8Set => deserializer
                .deserialize_set_extend_new::<tags::I8, i8, _>()
                .map(Self::I8Set),

            ValueKind::U16Set => deserializer
                .deserialize_set_extend_new::<tags::U16, u16, _>()
                .map(Self::U16Set),

            ValueKind::I16Set => deserializer
                .deserialize_set_extend_new::<tags::I16, i16, _>()
                .map(Self::I16Set),

            ValueKind::U32Set => deserializer
                .deserialize_set_extend_new::<tags::U32, u32, _>()
                .map(Self::U32Set),

            ValueKind::I32Set => deserializer
                .deserialize_set_extend_new::<tags::I32, i32, _>()
                .map(Self::I32Set),

            ValueKind::U64Set => deserializer
                .deserialize_set_extend_new::<tags::U64, u64, _>()
                .map(Self::U64Set),

            ValueKind::I64Set => deserializer
                .deserialize_set_extend_new::<tags::I64, i64, _>()
                .map(Self::I64Set),

            ValueKind::StringSet => deserializer
                .deserialize_set_extend_new::<tags::String, _, _>()
                .map(Self::StringSet),

            ValueKind::UuidSet => deserializer
                .deserialize_set_extend_new::<tags::Uuid, Uuid, _>()
                .map(Self::UuidSet),

            ValueKind::Struct => deserializer.deserialize().map(Self::Struct),
            ValueKind::Enum => deserializer.deserialize().map(Self::Enum),
            ValueKind::Sender => deserializer.deserialize_sender().map(Self::Sender),
            ValueKind::Receiver => deserializer.deserialize_receiver().map(Self::Receiver),
            ValueKind::Vec2 => deserializer.deserialize_vec2_extend_new().map(Self::Vec),

            ValueKind::Bytes2 => deserializer
                .deserialize_bytes2_extend_new()
                .map(Bytes)
                .map(Self::Bytes),

            ValueKind::U8Map2 => deserializer
                .deserialize_map2_extend_new::<tags::U8, _, _, _, _>()
                .map(Self::U8Map),

            ValueKind::I8Map2 => deserializer
                .deserialize_map2_extend_new::<tags::I8, _, _, _, _>()
                .map(Self::I8Map),

            ValueKind::U16Map2 => deserializer
                .deserialize_map2_extend_new::<tags::U16, _, _, _, _>()
                .map(Self::U16Map),

            ValueKind::I16Map2 => deserializer
                .deserialize_map2_extend_new::<tags::I16, _, _, _, _>()
                .map(Self::I16Map),

            ValueKind::U32Map2 => deserializer
                .deserialize_map2_extend_new::<tags::U32, _, _, _, _>()
                .map(Self::U32Map),

            ValueKind::I32Map2 => deserializer
                .deserialize_map2_extend_new::<tags::I32, _, _, _, _>()
                .map(Self::I32Map),

            ValueKind::U64Map2 => deserializer
                .deserialize_map2_extend_new::<tags::U64, _, _, _, _>()
                .map(Self::U64Map),

            ValueKind::I64Map2 => deserializer
                .deserialize_map2_extend_new::<tags::I64, _, _, _, _>()
                .map(Self::I64Map),

            ValueKind::StringMap2 => deserializer
                .deserialize_map2_extend_new::<tags::String, _, _, _, _>()
                .map(Self::StringMap),

            ValueKind::UuidMap2 => deserializer
                .deserialize_map2_extend_new::<tags::Uuid, _, _, _, _>()
                .map(Self::UuidMap),
        }
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for Value {
    fn layout() -> Layout {
        BuiltInType::Value.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::VALUE
    }

    fn add_references(_references: &mut References) {}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Struct(pub HashMap<u32, Value>);

impl Tag for Struct {}

impl PrimaryTag for Struct {
    type Tag = Self;
}

impl Serialize<Self> for Struct {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Deserialize<Self> for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut value = HashMap::new();
        while !deserializer.is_empty() {
            let deserializer = deserializer.deserialize()?;

            let id = deserializer.id();
            let field = deserializer.deserialize()?;

            value.insert(id, field);
        }

        deserializer.finish(Self(value))
    }
}

impl Serialize<Struct> for &Struct {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(self.0.len())?;

        for (&id, field) in &self.0 {
            serializer.serialize(id, field)?;
        }

        serializer.finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Enum {
    pub variant: u32,
    pub value: Value,
}

impl Enum {
    pub fn new(variant: u32, value: Value) -> Self {
        Self { variant, value }
    }
}

impl Tag for Enum {}

impl PrimaryTag for Enum {
    type Tag = Self;
}

impl Serialize<Self> for Enum {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Deserialize<Self> for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        let variant = deserializer.variant();
        let value = deserializer.deserialize()?;

        Ok(Self::new(variant, value))
    }
}

impl Serialize<Enum> for &Enum {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_enum(self.variant, &self.value)
    }
}
