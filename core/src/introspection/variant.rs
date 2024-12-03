use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variant {
    id: u32,
    name: String,
    variant_type: Option<LexicalId>,
}

impl Variant {
    pub(super) fn new(id: u32, name: impl Into<String>, variant_type: Option<LexicalId>) -> Self {
        Self {
            id,
            name: name.into(),
            variant_type,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variant_type(&self) -> Option<LexicalId> {
        self.variant_type
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

        let mut id = None;
        let mut name = None;
        let mut variant_type = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id()? {
                VariantField::Id => id = deserializer.deserialize().map(Some)?,
                VariantField::Name => name = deserializer.deserialize().map(Some)?,
                VariantField::VariantType => variant_type = deserializer.deserialize()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            variant_type,
        })
    }
}
