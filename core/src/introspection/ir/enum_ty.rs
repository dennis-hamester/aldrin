use super::{EnumFallbackIr, LexicalId, VariantIr};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{Uuid, uuid};

#[derive(Debug, Clone)]
pub struct EnumIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
    pub(crate) variants: BTreeMap<u32, VariantIr>,
    pub(crate) fallback: Option<EnumFallbackIr>,
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

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn variants(&self) -> &BTreeMap<u32, VariantIr> {
        &self.variants
    }

    pub fn fallback(&self) -> Option<&EnumFallbackIr> {
        self.fallback.as_ref()
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

impl Serialize<EnumIr> for &EnumIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(EnumField::Schema, &self.schema)?;
        serializer.serialize::<tags::String>(EnumField::Name, &self.name)?;

        serializer
            .serialize::<tags::Map<tags::U32, VariantIr>>(EnumField::Variants, &self.variants)?;

        serializer.serialize_if_some::<tags::Option<EnumFallbackIr>>(
            EnumField::Fallback,
            &self.fallback,
        )?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct EnumIrBuilder {
    schema: String,
    name: String,
    doc: Option<String>,
    variants: BTreeMap<u32, VariantIr>,
    fallback: Option<EnumFallbackIr>,
}

impl EnumIrBuilder {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            doc: None,
            variants: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn variant(mut self, variant: VariantIr) -> Self {
        self.variants.insert(variant.id(), variant);
        self
    }

    pub fn fallback(mut self, fallback: EnumFallbackIr) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn finish(self) -> EnumIr {
        EnumIr {
            schema: self.schema,
            name: self.name,
            doc: self.doc,
            variants: self.variants,
            fallback: self.fallback,
        }
    }
}
