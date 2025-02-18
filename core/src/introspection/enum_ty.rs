use super::{LexicalId, Variant};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enum {
    schema: String,
    name: String,
    variants: BTreeMap<u32, Variant>,
    fallback: Option<String>,
}

impl Enum {
    pub const NAMESPACE: Uuid = uuid!("642bf73e-991f-406a-b55a-ce914d77480b");

    pub fn builder(schema: impl Into<String>, name: impl Into<String>) -> EnumBuilder {
        EnumBuilder::new(schema, name)
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

    pub fn variants(&self) -> &BTreeMap<u32, Variant> {
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

impl Serialize for Enum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let num = 3 + (self.fallback.is_some() as usize);
        let mut serializer = serializer.serialize_struct(num)?;

        serializer.serialize_field(EnumField::Schema, &self.schema)?;
        serializer.serialize_field(EnumField::Name, &self.name)?;
        serializer.serialize_field(EnumField::Variants, &self.variants)?;

        if let Some(ref fallback) = self.fallback {
            serializer.serialize_field(EnumField::Fallback, &Some(fallback))?;
        }

        serializer.finish()
    }
}

impl Deserialize for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut variants = None;
        let mut fallback = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id()? {
                EnumField::Schema => schema = deserializer.deserialize().map(Some)?,
                EnumField::Name => name = deserializer.deserialize().map(Some)?,
                EnumField::Variants => variants = deserializer.deserialize().map(Some)?,
                EnumField::Fallback => fallback = deserializer.deserialize()?,
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
pub struct EnumBuilder {
    schema: String,
    name: String,
    variants: BTreeMap<u32, Variant>,
    fallback: Option<String>,
}

impl EnumBuilder {
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
            .insert(id, Variant::new(id, name, variant_type));
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

    pub fn finish(self) -> Enum {
        Enum {
            schema: self.schema,
            name: self.name,
            variants: self.variants,
            fallback: self.fallback,
        }
    }
}
