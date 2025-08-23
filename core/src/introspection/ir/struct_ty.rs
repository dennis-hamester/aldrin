use super::{FieldIr, LexicalId, StructFallbackIr};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct StructIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) fields: BTreeMap<u32, FieldIr>,
    pub(crate) fallback: Option<StructFallbackIr>,
}

impl StructIr {
    pub const NAMESPACE: Uuid = uuid!("83742d78-4e60-44b2-84e7-75904c5987c1");

    pub fn builder(schema: impl Into<String>, name: impl Into<String>) -> StructIrBuilder {
        StructIrBuilder::new(schema, name)
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

    pub fn fields(&self) -> &BTreeMap<u32, FieldIr> {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&StructFallbackIr> {
        self.fallback.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum StructField {
    Schema = 0,
    Name = 1,
    Fields = 2,
    Fallback = 3,
}

impl Tag for StructIr {}

impl PrimaryTag for StructIr {
    type Tag = Self;
}

impl Serialize<StructIr> for &StructIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(StructField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(StructField::Name, &self.name)?;

        serializer
            .serialize::<tags::Map<tags::U32, FieldIr>, _>(StructField::Fields, &self.fields)?;

        serializer.serialize_if_some::<tags::Option<StructFallbackIr>, _>(
            StructField::Fallback,
            &self.fallback,
        )?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct StructIrBuilder {
    schema: String,
    name: String,
    fields: BTreeMap<u32, FieldIr>,
    fallback: Option<StructFallbackIr>,
}

impl StructIrBuilder {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            fields: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn field(
        mut self,
        id: u32,
        name: impl Into<String>,
        is_required: bool,
        field_type: LexicalId,
    ) -> Self {
        self.fields
            .insert(id, FieldIr::new(id, name, is_required, field_type));
        self
    }

    pub fn fallback(mut self, fallback: StructFallbackIr) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn finish(self) -> StructIr {
        StructIr {
            schema: self.schema,
            name: self.name,
            fields: self.fields,
            fallback: self.fallback,
        }
    }
}
