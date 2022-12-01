use crate::error::{DeserializeError, SerializeError};
use crate::ids::{ChannelCookie, ObjectId, ServiceId};
use crate::value::ValueKind;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Tests are in crate::value::test;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Some(Box<Value>),
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
    Vec(Vec<Value>),
    Bytes(Vec<u8>),
    U8Map(HashMap<u8, Value>),
    I8Map(HashMap<i8, Value>),
    U16Map(HashMap<u16, Value>),
    I16Map(HashMap<i16, Value>),
    U32Map(HashMap<u32, Value>),
    I32Map(HashMap<i32, Value>),
    U64Map(HashMap<u64, Value>),
    I64Map(HashMap<i64, Value>),
    StringMap(HashMap<String, Value>),
    UuidMap(HashMap<Uuid, Value>),
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

    pub fn kind(&self) -> ValueKind {
        match self {
            Self::None => ValueKind::None,
            Self::Some(_) => ValueKind::Some,
            Self::Bool(_) => ValueKind::Bool,
            Self::U8(_) => ValueKind::U8,
            Self::I8(_) => ValueKind::I8,
            Self::U16(_) => ValueKind::U16,
            Self::I16(_) => ValueKind::I16,
            Self::U32(_) => ValueKind::U32,
            Self::I32(_) => ValueKind::I32,
            Self::U64(_) => ValueKind::U64,
            Self::I64(_) => ValueKind::I64,
            Self::F32(_) => ValueKind::F32,
            Self::F64(_) => ValueKind::F64,
            Self::String(_) => ValueKind::String,
            Self::Uuid(_) => ValueKind::Uuid,
            Self::ObjectId(_) => ValueKind::ObjectId,
            Self::ServiceId(_) => ValueKind::ServiceId,
            Self::Vec(_) => ValueKind::Vec,
            Self::Bytes(_) => ValueKind::Bytes,
            Self::U8Map(_) => ValueKind::U8Map,
            Self::I8Map(_) => ValueKind::I8Map,
            Self::U16Map(_) => ValueKind::U16Map,
            Self::I16Map(_) => ValueKind::I16Map,
            Self::U32Map(_) => ValueKind::U32Map,
            Self::I32Map(_) => ValueKind::I32Map,
            Self::U64Map(_) => ValueKind::U64Map,
            Self::I64Map(_) => ValueKind::I64Map,
            Self::StringMap(_) => ValueKind::StringMap,
            Self::UuidMap(_) => ValueKind::UuidMap,
            Self::U8Set(_) => ValueKind::U8Set,
            Self::I8Set(_) => ValueKind::I8Set,
            Self::U16Set(_) => ValueKind::U16Set,
            Self::I16Set(_) => ValueKind::I16Set,
            Self::U32Set(_) => ValueKind::U32Set,
            Self::I32Set(_) => ValueKind::I32Set,
            Self::U64Set(_) => ValueKind::U64Set,
            Self::I64Set(_) => ValueKind::I64Set,
            Self::StringSet(_) => ValueKind::StringSet,
            Self::UuidSet(_) => ValueKind::UuidSet,
            Self::Struct(_) => ValueKind::Struct,
            Self::Enum(_) => ValueKind::Enum,
            Self::Sender(_) => ValueKind::Sender,
            Self::Receiver(_) => ValueKind::Receiver,
        }
    }
}

impl Serialize for Value {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::None => serializer.serialize_none(),
            Self::Some(value) => serializer.serialize_some(value),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::U8(value) => serializer.serialize_u8(*value),
            Self::I8(value) => serializer.serialize_i8(*value),
            Self::U16(value) => serializer.serialize_u16(*value),
            Self::I16(value) => serializer.serialize_i16(*value),
            Self::U32(value) => serializer.serialize_u32(*value),
            Self::I32(value) => serializer.serialize_i32(*value),
            Self::U64(value) => serializer.serialize_u64(*value),
            Self::I64(value) => serializer.serialize_i64(*value),
            Self::F32(value) => serializer.serialize_f32(*value),
            Self::F64(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_string(value),
            Self::Uuid(value) => serializer.serialize_uuid(*value),
            Self::ObjectId(value) => serializer.serialize_object_id(*value),
            Self::ServiceId(value) => serializer.serialize_service_id(*value),
            Self::Vec(value) => serializer.serialize_vec_iter(value),
            Self::Bytes(value) => serializer.serialize_bytes(value),
            Self::U8Map(value) => serializer.serialize_map_iter(value),
            Self::I8Map(value) => serializer.serialize_map_iter(value),
            Self::U16Map(value) => serializer.serialize_map_iter(value),
            Self::I16Map(value) => serializer.serialize_map_iter(value),
            Self::U32Map(value) => serializer.serialize_map_iter(value),
            Self::I32Map(value) => serializer.serialize_map_iter(value),
            Self::U64Map(value) => serializer.serialize_map_iter(value),
            Self::I64Map(value) => serializer.serialize_map_iter(value),
            Self::StringMap(value) => serializer.serialize_map_iter(value),
            Self::UuidMap(value) => serializer.serialize_map_iter(value),
            Self::U8Set(value) => serializer.serialize_set_iter(value),
            Self::I8Set(value) => serializer.serialize_set_iter(value),
            Self::U16Set(value) => serializer.serialize_set_iter(value),
            Self::I16Set(value) => serializer.serialize_set_iter(value),
            Self::U32Set(value) => serializer.serialize_set_iter(value),
            Self::I32Set(value) => serializer.serialize_set_iter(value),
            Self::U64Set(value) => serializer.serialize_set_iter(value),
            Self::I64Set(value) => serializer.serialize_set_iter(value),
            Self::StringSet(value) => serializer.serialize_set_iter(value),
            Self::UuidSet(value) => serializer.serialize_set_iter(value),
            Self::Struct(value) => value.serialize(serializer),
            Self::Enum(value) => value.serialize(serializer),
            Self::Sender(value) => serializer.serialize_sender(*value),
            Self::Receiver(value) => serializer.serialize_receiver(*value),
        }
    }
}

impl Deserialize for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|_| Self::None),
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
            ValueKind::Vec => deserializer.deserialize_vec_extend_new().map(Self::Vec),
            ValueKind::Bytes => deserializer.deserialize_bytes_to_vec().map(Self::Bytes),
            ValueKind::U8Map => deserializer.deserialize_map_extend_new().map(Self::U8Map),
            ValueKind::I8Map => deserializer.deserialize_map_extend_new().map(Self::I8Map),
            ValueKind::U16Map => deserializer.deserialize_map_extend_new().map(Self::U16Map),
            ValueKind::I16Map => deserializer.deserialize_map_extend_new().map(Self::I16Map),
            ValueKind::U32Map => deserializer.deserialize_map_extend_new().map(Self::U32Map),
            ValueKind::I32Map => deserializer.deserialize_map_extend_new().map(Self::I32Map),
            ValueKind::U64Map => deserializer.deserialize_map_extend_new().map(Self::U64Map),
            ValueKind::I64Map => deserializer.deserialize_map_extend_new().map(Self::I64Map),
            ValueKind::StringMap => deserializer
                .deserialize_map_extend_new()
                .map(Self::StringMap),
            ValueKind::UuidMap => deserializer.deserialize_map_extend_new().map(Self::UuidMap),
            ValueKind::U8Set => deserializer
                .deserialize_set_extend_new::<u8, _>()
                .map(Self::U8Set),
            ValueKind::I8Set => deserializer
                .deserialize_set_extend_new::<i8, _>()
                .map(Self::I8Set),
            ValueKind::U16Set => deserializer
                .deserialize_set_extend_new::<u16, _>()
                .map(Self::U16Set),
            ValueKind::I16Set => deserializer
                .deserialize_set_extend_new::<i16, _>()
                .map(Self::I16Set),
            ValueKind::U32Set => deserializer
                .deserialize_set_extend_new::<u32, _>()
                .map(Self::U32Set),
            ValueKind::I32Set => deserializer
                .deserialize_set_extend_new::<i32, _>()
                .map(Self::I32Set),
            ValueKind::U64Set => deserializer
                .deserialize_set_extend_new::<u64, _>()
                .map(Self::U64Set),
            ValueKind::I64Set => deserializer
                .deserialize_set_extend_new::<i64, _>()
                .map(Self::I64Set),
            ValueKind::StringSet => deserializer
                .deserialize_set_extend_new()
                .map(Self::StringSet),
            ValueKind::UuidSet => deserializer
                .deserialize_set_extend_new::<Uuid, _>()
                .map(Self::UuidSet),
            ValueKind::Struct => Struct::deserialize(deserializer).map(Self::Struct),
            ValueKind::Enum => Enum::deserialize(deserializer)
                .map(Box::new)
                .map(Self::Enum),
            ValueKind::Sender => deserializer.deserialize_sender().map(Self::Sender),
            ValueKind::Receiver => deserializer.deserialize_receiver().map(Self::Receiver),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct(pub HashMap<u32, Value>);

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let num_fields = self.0.values().filter(|v| !v.is_none()).count();
        let mut serializer = serializer.serialize_struct(num_fields)?;

        for (&id, field) in &self.0 {
            if !field.is_none() {
                serializer.serialize_field(id, field)?;
            }
        }

        serializer.finish()
    }
}

impl Deserialize for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut value = HashMap::new();
        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;
            let id = deserializer.id();
            let field: Value = deserializer.deserialize()?;
            if !field.is_none() {
                value.insert(id, field);
            }
        }

        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub variant: u32,
    pub value: Value,
}

impl Enum {
    pub fn new(variant: u32, value: Value) -> Self {
        Self { variant, value }
    }
}

impl Serialize for Enum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_enum(self.variant, &self.value)
    }
}

impl Deserialize for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;
        let variant = deserializer.variant();
        let value = deserializer.deserialize()?;
        Ok(Self::new(variant, value))
    }
}
