use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CustomType {
    schema: String,
    name: String,
}

impl CustomType {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum CustomTypeField {
    Schema = 0,
    Name = 1,
}

impl Serialize for CustomType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(CustomTypeField::Schema, &self.schema)?;
        serializer.serialize_field(CustomTypeField::Name, &self.name)?;

        serializer.finish()
    }
}

impl Deserialize for CustomType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let schema = deserializer.deserialize_specific_field(CustomTypeField::Schema)?;
        let name = deserializer.deserialize_specific_field(CustomTypeField::Name)?;

        deserializer.finish(Self { schema, name })
    }
}

impl fmt::Display for CustomType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.schema, self.name)
    }
}
