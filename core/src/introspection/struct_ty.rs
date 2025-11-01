use super::{ir, Field, LexicalId, StructFallback};
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
pub struct Struct {
    schema: String,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    fields: BTreeMap<u32, Field>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    fallback: Option<StructFallback>,
}

impl Struct {
    pub fn from_ir(ty: ir::StructIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            schema: ty.schema,
            name: ty.name,
            doc: ty.doc,
            fields: ty
                .fields
                .into_iter()
                .map(|(id, field)| (id, Field::from_ir(field, references)))
                .collect(),
            fallback: ty.fallback.map(StructFallback::from_ir),
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

    pub fn fields(&self) -> &BTreeMap<u32, Field> {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&StructFallback> {
        self.fallback.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum StructField {
    Schema = 0,
    Name = 1,
    Doc = 2,
    Fields = 3,
    Fallback = 4,
}

impl Tag for Struct {}

impl PrimaryTag for Struct {
    type Tag = Self;
}

impl Serialize<Self> for Struct {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Struct> for &Struct {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(StructField::Schema, &self.schema)?;
        serializer.serialize::<tags::String>(StructField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(StructField::Doc, &self.doc)?;
        serializer.serialize::<tags::Map<tags::U32, Field>>(StructField::Fields, &self.fields)?;

        serializer.serialize_if_some::<tags::Option<StructFallback>>(
            StructField::Fallback,
            &self.fallback,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut doc = None;
        let mut fields = None;
        let mut fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(StructField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(StructField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(StructField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(StructField::Fields) => {
                    fields = deserializer
                        .deserialize::<tags::Map<tags::U32, Field>, _>()
                        .map(Some)?;
                }

                Ok(StructField::Fallback) => {
                    fallback = deserializer.deserialize::<tags::Option<StructFallback>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            fields: fields.ok_or(DeserializeError::InvalidSerialization)?,
            fallback,
        })
    }
}
