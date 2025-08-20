use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariantIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) variant_type: Option<LexicalId>,
}

impl VariantIr {
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

impl Tag for VariantIr {}

impl PrimaryTag for VariantIr {
    type Tag = Self;
}

impl Serialize<Self> for VariantIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<VariantIr> for &VariantIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(VariantField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(VariantField::Name, &self.name)?;

        serializer.serialize::<tags::Option<LexicalId>, _>(
            VariantField::VariantType,
            self.variant_type,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for VariantIr {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut variant_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(VariantField::Id) => {
                    id = deserializer.deserialize::<tags::U32, _>().map(Some)?
                }

                Ok(VariantField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(VariantField::VariantType) => {
                    variant_type = deserializer.deserialize::<tags::Option<LexicalId>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            variant_type,
        })
    }
}
