use super::{ir, EnumFallback, LexicalId, Variant};
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
pub struct Enum {
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
    variants: BTreeMap<u32, Variant>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    fallback: Option<EnumFallback>,
}

impl Enum {
    pub fn from_ir(ty: ir::EnumIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            schema: ty.schema,
            name: ty.name,
            doc: ty.doc,
            variants: ty
                .variants
                .into_iter()
                .map(|(id, var)| (id, Variant::from_ir(var, references)))
                .collect(),
            fallback: ty.fallback.map(EnumFallback::from_ir),
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

    pub fn variants(&self) -> &BTreeMap<u32, Variant> {
        &self.variants
    }

    pub fn fallback(&self) -> Option<&EnumFallback> {
        self.fallback.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EnumField {
    Schema = 0,
    Name = 1,
    Doc = 2,
    Variants = 3,
    Fallback = 4,
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

impl Serialize<Enum> for &Enum {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(EnumField::Schema, &self.schema)?;
        serializer.serialize::<tags::String>(EnumField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(EnumField::Doc, &self.doc)?;

        serializer
            .serialize::<tags::Map<tags::U32, Variant>>(EnumField::Variants, &self.variants)?;

        serializer
            .serialize_if_some::<tags::Option<EnumFallback>>(EnumField::Fallback, &self.fallback)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut doc = None;
        let mut variants = None;
        let mut fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EnumField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(EnumField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(EnumField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(EnumField::Variants) => {
                    variants = deserializer
                        .deserialize::<tags::Map<tags::U32, Variant>, _>()
                        .map(Some)?;
                }

                Ok(EnumField::Fallback) => {
                    fallback = deserializer.deserialize::<tags::Option<EnumFallback>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            variants: variants.ok_or(DeserializeError::InvalidSerialization)?,
            fallback,
        })
    }
}
