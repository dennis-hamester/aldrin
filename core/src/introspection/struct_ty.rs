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
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum StructField {
    Schema = 0,
    Name = 1,
    Fields = 2,
}

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(StructField::Schema, &self.schema)?;
        serializer.serialize_field(StructField::Name, &self.name)?;
        serializer.serialize_field(StructField::Fields, &self.fields)?;

        serializer.finish()
    }
}

impl Deserialize for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let schema = deserializer.deserialize_specific_field(StructField::Schema)?;
        let name = deserializer.deserialize_specific_field(StructField::Name)?;
        let fields = deserializer.deserialize_specific_field(StructField::Fields)?;

        deserializer.finish(Self {
            schema,
            name,
            fields,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructBuilder {
    schema: String,
    name: String,
    fields: BTreeMap<u32, Field>,
}

impl StructBuilder {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            fields: BTreeMap::new(),
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

    pub fn finish(self) -> Struct {
        Struct {
            schema: self.schema,
            name: self.name,
            fields: self.fields,
        }
    }
}
