use super::{Field, LexicalId};
use crate::adapters::IterAsMap1;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
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
        let num = 3 + (self.fallback.is_some() as usize);
        let mut serializer = serializer.serialize_struct1(num)?;

        serializer.serialize::<tags::String, _>(StructField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(StructField::Name, &self.name)?;

        serializer.serialize::<tags::Map<tags::U32, Field>, _>(
            StructField::Fields,
            IterAsMap1(&self.fields),
        )?;

        serializer.serialize_if_some::<tags::Option<tags::String>, _>(
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
        let mut fields = None;
        let mut fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(StructField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?
                }
                Ok(StructField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(StructField::Fields) => {
                    fields = deserializer
                        .deserialize::<tags::Map<tags::U32, Field>, _>()
                        .map(Some)?
                }

                Ok(StructField::Fallback) => {
                    fallback = deserializer.deserialize::<tags::Option<tags::String>, _>()?
                }

                Err(_) => deserializer.skip()?,
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
