use super::TypeRef;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct Variant {
    id: u32,
    name: String,
    variant_type: Option<TypeRef>,
}

impl Variant {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variant_type(&self) -> Option<&TypeRef> {
        self.variant_type.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum VariantField {
    Id = 0,
    Name = 1,
    VariantType = 2,
}

impl Serialize for Variant {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(VariantField::Id, &self.id)?;
        serializer.serialize_field(VariantField::Name, &self.name)?;
        serializer.serialize_field(VariantField::VariantType, &self.variant_type)?;

        serializer.finish()
    }
}

impl Deserialize for Variant {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_specific_field(VariantField::Id)?;
        let name = deserializer.deserialize_specific_field(VariantField::Name)?;
        let variant_type = deserializer.deserialize_specific_field(VariantField::VariantType)?;

        deserializer.finish(Self {
            id,
            name,
            variant_type,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VariantBuilder {
    id: u32,
    name: String,
    variant_type: Option<TypeRef>,
}

impl VariantBuilder {
    pub(crate) fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            variant_type: None,
        }
    }

    pub fn variant_type(mut self, variant_type: impl Into<TypeRef>) -> Self {
        self.variant_type = Some(variant_type.into());
        self
    }

    pub fn finish(self) -> Variant {
        Variant {
            id: self.id,
            name: self.name,
            variant_type: self.variant_type,
        }
    }
}
