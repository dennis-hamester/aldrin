use crate::error::{DeserializeError, SerializeError};
use crate::ids::{ChannelCookie, ObjectId, ServiceId};
#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::value::{Bytes, ValueKind};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{AsSerializeArg, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

impl Serialize for Value {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::None => serializer.serialize_none(),
            Self::Some(value) => serializer.serialize_some(value)?,
            Self::Bool(value) => serializer.serialize(value)?,
            Self::U8(value) => serializer.serialize(value)?,
            Self::I8(value) => serializer.serialize(value)?,
            Self::U16(value) => serializer.serialize(value)?,
            Self::I16(value) => serializer.serialize(value)?,
            Self::U32(value) => serializer.serialize(value)?,
            Self::I32(value) => serializer.serialize(value)?,
            Self::U64(value) => serializer.serialize(value)?,
            Self::I64(value) => serializer.serialize(value)?,
            Self::F32(value) => serializer.serialize(value)?,
            Self::F64(value) => serializer.serialize(value)?,
            Self::String(value) => serializer.serialize(value)?,
            Self::Uuid(value) => serializer.serialize(value)?,
            Self::ObjectId(value) => serializer.serialize(value)?,
            Self::ServiceId(value) => serializer.serialize(value)?,
            Self::Vec(value) => serializer.serialize(value)?,
            Self::Bytes(value) => serializer.serialize(value)?,
            Self::U8Map(value) => serializer.serialize(value)?,
            Self::I8Map(value) => serializer.serialize(value)?,
            Self::U16Map(value) => serializer.serialize(value)?,
            Self::I16Map(value) => serializer.serialize(value)?,
            Self::U32Map(value) => serializer.serialize(value)?,
            Self::I32Map(value) => serializer.serialize(value)?,
            Self::U64Map(value) => serializer.serialize(value)?,
            Self::I64Map(value) => serializer.serialize(value)?,
            Self::StringMap(value) => serializer.serialize(value)?,
            Self::UuidMap(value) => serializer.serialize(value)?,
            Self::U8Set(value) => serializer.serialize(value)?,
            Self::I8Set(value) => serializer.serialize(value)?,
            Self::U16Set(value) => serializer.serialize(value)?,
            Self::I16Set(value) => serializer.serialize(value)?,
            Self::U32Set(value) => serializer.serialize(value)?,
            Self::I32Set(value) => serializer.serialize(value)?,
            Self::U64Set(value) => serializer.serialize(value)?,
            Self::I64Set(value) => serializer.serialize(value)?,
            Self::StringSet(value) => serializer.serialize(value)?,
            Self::UuidSet(value) => serializer.serialize(value)?,
            Self::Struct(value) => serializer.serialize(value)?,
            Self::Enum(value) => serializer.serialize(value)?,
            Self::Sender(value) => serializer.serialize_sender(*value),
            Self::Receiver(value) => serializer.serialize_receiver(*value),
        }

        Ok(())
    }
}

impl Deserialize for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|()| Self::None),
            ValueKind::Some => deserializer.deserialize_some().map(Self::Some),
            ValueKind::Bool => deserializer.deserialize().map(Self::Bool),
            ValueKind::U8 => deserializer.deserialize().map(Self::U8),
            ValueKind::I8 => deserializer.deserialize().map(Self::I8),
            ValueKind::U16 => deserializer.deserialize().map(Self::U16),
            ValueKind::I16 => deserializer.deserialize().map(Self::I16),
            ValueKind::U32 => deserializer.deserialize().map(Self::U32),
            ValueKind::I32 => deserializer.deserialize().map(Self::I32),
            ValueKind::U64 => deserializer.deserialize().map(Self::U64),
            ValueKind::I64 => deserializer.deserialize().map(Self::I64),
            ValueKind::F32 => deserializer.deserialize().map(Self::F32),
            ValueKind::F64 => deserializer.deserialize().map(Self::F64),
            ValueKind::String => deserializer.deserialize().map(Self::String),
            ValueKind::Uuid => deserializer.deserialize().map(Self::Uuid),
            ValueKind::ObjectId => deserializer.deserialize().map(Self::ObjectId),
            ValueKind::ServiceId => deserializer.deserialize().map(Self::ServiceId),
            ValueKind::Vec => deserializer.deserialize().map(Self::Vec),
            ValueKind::Bytes => deserializer.deserialize().map(Self::Bytes),
            ValueKind::U8Map => deserializer.deserialize().map(Self::U8Map),
            ValueKind::I8Map => deserializer.deserialize().map(Self::I8Map),
            ValueKind::U16Map => deserializer.deserialize().map(Self::U16Map),
            ValueKind::I16Map => deserializer.deserialize().map(Self::I16Map),
            ValueKind::U32Map => deserializer.deserialize().map(Self::U32Map),
            ValueKind::I32Map => deserializer.deserialize().map(Self::I32Map),
            ValueKind::U64Map => deserializer.deserialize().map(Self::U64Map),
            ValueKind::I64Map => deserializer.deserialize().map(Self::I64Map),
            ValueKind::StringMap => deserializer.deserialize().map(Self::StringMap),
            ValueKind::UuidMap => deserializer.deserialize().map(Self::UuidMap),
            ValueKind::U8Set => deserializer.deserialize().map(Self::U8Set),
            ValueKind::I8Set => deserializer.deserialize().map(Self::I8Set),
            ValueKind::U16Set => deserializer.deserialize().map(Self::U16Set),
            ValueKind::I16Set => deserializer.deserialize().map(Self::I16Set),
            ValueKind::U32Set => deserializer.deserialize().map(Self::U32Set),
            ValueKind::I32Set => deserializer.deserialize().map(Self::I32Set),
            ValueKind::U64Set => deserializer.deserialize().map(Self::U64Set),
            ValueKind::I64Set => deserializer.deserialize().map(Self::I64Set),
            ValueKind::StringSet => deserializer.deserialize().map(Self::StringSet),
            ValueKind::UuidSet => deserializer.deserialize().map(Self::UuidSet),
            ValueKind::Struct => deserializer.deserialize().map(Self::Struct),
            ValueKind::Enum => deserializer.deserialize().map(Self::Enum),
            ValueKind::Sender => deserializer.deserialize_sender().map(Self::Sender),
            ValueKind::Receiver => deserializer.deserialize_receiver().map(Self::Receiver),
        }
    }
}

impl AsSerializeArg for Value {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Struct(pub HashMap<u32, Value>);

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(self.0.len())?;

        for (&id, field) in &self.0 {
            serializer.serialize_field(id, field)?;
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
            let field = deserializer.deserialize()?;
            value.insert(id, field);
        }

        deserializer.finish(Self(value))
    }
}

impl AsSerializeArg for Struct {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
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

impl AsSerializeArg for Enum {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}
