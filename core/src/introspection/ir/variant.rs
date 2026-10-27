use super::LexicalId;
use crate::enum_repr::enum_with_repr;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};

#[derive(Debug, Clone)]
pub struct VariantIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
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

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn variant_type(&self) -> Option<LexicalId> {
        self.variant_type
    }
}

enum_with_repr! {
    enum VariantField: u32 {
        Id = 0,
        Name = 1,
        VariantType = 2,
    }
}

impl Tag for VariantIr {}

impl PrimaryTag for VariantIr {
    type Tag = Self;
}

impl Serialize<VariantIr> for &VariantIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32>(VariantField::Id, &self.id)?;
        serializer.serialize::<tags::String>(VariantField::Name, &self.name)?;

        serializer.serialize_if_some::<tags::Option<LexicalId>>(
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
    doc: Option<String>,
    variant_type: Option<LexicalId>,
}

impl VariantIrBuilder {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            doc: None,
            variant_type: None,
        }
    }

    #[must_use]
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    #[must_use]
    pub fn variant_type(mut self, variant_type: LexicalId) -> Self {
        self.variant_type = Some(variant_type);
        self
    }

    pub fn finish(self) -> VariantIr {
        VariantIr {
            id: self.id,
            name: self.name,
            doc: self.doc,
            variant_type: self.variant_type,
        }
    }
}
