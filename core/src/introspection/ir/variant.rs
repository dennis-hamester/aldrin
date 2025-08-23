use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct VariantIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) variant_type: Option<LexicalId>,
}

impl VariantIr {
    pub fn builder(id: u32, name: impl Into<String>) -> VariantIrBuilder {
        VariantIrBuilder::new(id, name)
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

impl Serialize<VariantIr> for &VariantIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(VariantField::Id, &self.id)?;
        serializer.serialize::<tags::String, _>(VariantField::Name, &self.name)?;

        serializer.serialize_if_some::<tags::Option<LexicalId>, _>(
            VariantField::VariantType,
            &self.variant_type,
        )?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct VariantIrBuilder {
    id: u32,
    name: String,
    variant_type: Option<LexicalId>,
}

impl VariantIrBuilder {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            variant_type: None,
        }
    }

    pub fn variant_type(mut self, variant_type: LexicalId) -> Self {
        self.variant_type = Some(variant_type);
        self
    }

    pub fn finish(self) -> VariantIr {
        VariantIr {
            id: self.id,
            name: self.name,
            variant_type: self.variant_type,
        }
    }
}
