use super::{LexicalId, VariantIr};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) variants: BTreeMap<u32, VariantIr>,
    pub(crate) fallback: Option<String>,
}

impl EnumIr {
    pub const NAMESPACE: Uuid = uuid!("642bf73e-991f-406a-b55a-ce914d77480b");

    pub fn builder(schema: impl Into<String>, name: impl Into<String>) -> EnumIrBuilder {
        EnumIrBuilder::new(schema, name)
    }

    pub fn lexical_id(&self) -> LexicalId {
        LexicalId::custom(&self.schema, &self.name)
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variants(&self) -> &BTreeMap<u32, VariantIr> {
        &self.variants
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EnumField {
    Schema = 0,
    Name = 1,
    Variants = 2,
    Fallback = 3,
}

impl Tag for EnumIr {}

impl PrimaryTag for EnumIr {
    type Tag = Self;
}

impl Serialize<Self> for EnumIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<EnumIr> for &EnumIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(EnumField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(EnumField::Name, &self.name)?;

        serializer
            .serialize::<tags::Map<tags::U32, VariantIr>, _>(EnumField::Variants, &self.variants)?;

        serializer.serialize_if_some::<tags::Option<tags::String>, _>(
            EnumField::Fallback,
            &self.fallback,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for EnumIr {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut variants = None;
        let mut fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EnumField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(EnumField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(EnumField::Variants) => {
                    variants = deserializer
                        .deserialize::<tags::Map<tags::U32, VariantIr>, _>()
                        .map(Some)?
                }

                Ok(EnumField::Fallback) => {
                    fallback = deserializer.deserialize::<tags::Option<tags::String>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            variants: variants.ok_or(DeserializeError::InvalidSerialization)?,
            fallback,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EnumIrBuilder {
    schema: String,
    name: String,
    variants: BTreeMap<u32, VariantIr>,
    fallback: Option<String>,
}

impl EnumIrBuilder {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            variants: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn variant(
        mut self,
        id: u32,
        name: impl Into<String>,
        variant_type: Option<LexicalId>,
    ) -> Self {
        self.variants
            .insert(id, VariantIr::new(id, name, variant_type));
        self
    }

    pub fn variant_with_type(
        self,
        id: u32,
        name: impl Into<String>,
        variant_type: LexicalId,
    ) -> Self {
        self.variant(id, name, Some(variant_type))
    }

    pub fn unit_variant(self, id: u32, name: impl Into<String>) -> Self {
        self.variant(id, name, None)
    }

    pub fn fallback(mut self, name: impl Into<String>) -> Self {
        self.fallback = Some(name.into());
        self
    }

    pub fn finish(self) -> EnumIr {
        EnumIr {
            schema: self.schema,
            name: self.name,
            variants: self.variants,
            fallback: self.fallback,
        }
    }
}
