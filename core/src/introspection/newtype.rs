use super::{ir, resolve_ir, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Newtype {
    schema: String,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    target_type: TypeId,
}

impl Newtype {
    pub fn from_ir(ty: ir::NewtypeIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            schema: ty.schema,
            name: ty.name,
            doc: ty.doc,
            target_type: resolve_ir(ty.target_type, references),
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn target_type(&self) -> TypeId {
        self.target_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum NewtypeField {
    Schema = 0,
    Name = 1,
    Doc = 2,
    TargetType = 3,
}

impl Tag for Newtype {}

impl PrimaryTag for Newtype {
    type Tag = Self;
}

impl Serialize<Self> for Newtype {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Newtype> for &Newtype {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(NewtypeField::Schema, &self.schema)?;
        serializer.serialize::<tags::String>(NewtypeField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(NewtypeField::Doc, &self.doc)?;
        serializer.serialize::<TypeId>(NewtypeField::TargetType, &self.target_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Newtype {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut doc = None;
        let mut target_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(NewtypeField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(NewtypeField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(NewtypeField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(NewtypeField::TargetType) => {
                    target_type = deserializer.deserialize::<TypeId, _>().map(Some)?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            target_type: target_type.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
