use super::{Field, LexicalId};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Struct {
    schema: String,
    name: String,
    fields: BTreeMap<u32, Field>,
    fallback: Option<String>,
}

impl Struct {
    pub const NAMESPACE: Uuid = uuid!("83742d78-4e60-44b2-84e7-75904c5987c1");

    pub fn builder(schema: impl Into<String>, name: impl Into<String>) -> StructBuilder {
        StructBuilder::new(schema, name)
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

    pub fn fields(&self) -> &BTreeMap<u32, Field> {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
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

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let num = 3 + (self.fallback.is_some() as usize);
        let mut serializer = serializer.serialize_struct(num)?;

        serializer.serialize_field(StructField::Schema, &self.schema)?;
        serializer.serialize_field(StructField::Name, &self.name)?;
        serializer.serialize_field(StructField::Fields, &self.fields)?;

        if self.fallback.is_some() {
            serializer.serialize_field(StructField::Fallback, &self.fallback)?;
        }

        serializer.finish()
    }
}

impl Deserialize for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut fields = None;
        let mut fallback = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id()? {
                StructField::Schema => schema = deserializer.deserialize().map(Some)?,
                StructField::Name => name = deserializer.deserialize().map(Some)?,
                StructField::Fields => fields = deserializer.deserialize().map(Some)?,
                StructField::Fallback => fallback = deserializer.deserialize()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            fields: fields.ok_or(DeserializeError::InvalidSerialization)?,
            fallback,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructBuilder {
    schema: String,
    name: String,
    fields: BTreeMap<u32, Field>,
    fallback: Option<String>,
}

impl StructBuilder {
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
            .insert(id, Field::new(id, name, is_required, field_type));
        self
    }

    pub fn fallback(mut self, name: impl Into<String>) -> Self {
        self.fallback = Some(name.into());
        self
    }

    pub fn finish(self) -> Struct {
        Struct {
            schema: self.schema,
            name: self.name,
            fields: self.fields,
            fallback: self.fallback,
        }
    }
}
