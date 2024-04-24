use super::{Field, Layout, TypeRef};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct Struct {
    name: String,
    fields: BTreeMap<u32, Field>,
}

impl Struct {
    pub const NAMESPACE: Uuid = uuid!("83742d78-4e60-44b2-84e7-75904c5987c1");

    pub fn builder(name: impl Into<String>) -> StructBuilder {
        StructBuilder::new(name)
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
    Name = 0,
    Fields = 1,
}

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(StructField::Name, &self.name)?;
        serializer.serialize_field(StructField::Fields, &self.fields)?;

        serializer.finish()
    }
}

impl Deserialize for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let name = deserializer.deserialize_specific_field(StructField::Name)?;
        let fields = deserializer.deserialize_specific_field(StructField::Fields)?;

        deserializer.finish(Self { name, fields })
    }
}

impl From<Struct> for Layout {
    fn from(s: Struct) -> Self {
        Self::Struct(s)
    }
}

#[derive(Debug, Clone)]
pub struct StructBuilder {
    name: String,
    fields: BTreeMap<u32, Field>,
}

impl StructBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: BTreeMap::new(),
        }
    }

    pub fn field(
        mut self,
        id: u32,
        name: impl Into<String>,
        is_required: bool,
        data: impl Into<TypeRef>,
    ) -> Self {
        self.fields
            .insert(id, Field::new(id, name, is_required, data));
        self
    }

    pub fn finish(self) -> Struct {
        Struct {
            name: self.name,
            fields: self.fields,
        }
    }
}
