use super::{ir, resolve_ir, LexicalId};
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
pub struct MapType {
    key: TypeId,
    value: TypeId,
}

impl MapType {
    pub fn from_ir(ty: ir::MapTypeIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            key: resolve_ir(ty.key, references),
            value: resolve_ir(ty.value, references),
        }
    }

    pub fn key(self) -> TypeId {
        self.key
    }

    pub fn value(self) -> TypeId {
        self.value
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MapTypeField {
    Key = 0,
    Value = 1,
}

impl Tag for MapType {}

impl PrimaryTag for MapType {
    type Tag = Self;
}

impl Serialize<Self> for MapType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<TypeId, _>(MapTypeField::Key, self.key)?;
        serializer.serialize::<TypeId, _>(MapTypeField::Value, self.value)?;

        serializer.finish()
    }
}

impl Serialize<MapType> for &MapType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for MapType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut key = None;
        let mut value = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(MapTypeField::Key) => key = deserializer.deserialize::<TypeId, _>().map(Some)?,

                Ok(MapTypeField::Value) => {
                    value = deserializer.deserialize::<TypeId, _>().map(Some)?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            key: key.ok_or(DeserializeError::InvalidSerialization)?,
            value: value.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
