use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct FieldIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
    pub(crate) is_required: bool,
    pub(crate) field_type: LexicalId,
}

impl FieldIr {
    pub fn builder(
        id: u32,
        name: impl Into<String>,
        is_required: bool,
        field_type: LexicalId,
    ) -> FieldIrBuilder {
        FieldIrBuilder::new(id, name, is_required, field_type)
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

    pub fn is_required(&self) -> bool {
        self.is_required
    }

    pub fn field_type(&self) -> LexicalId {
        self.field_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FieldField {
    Id = 0,
    Name = 1,
    IsRequired = 2,
    FieldType = 3,
}

impl Tag for FieldIr {}

impl PrimaryTag for FieldIr {
    type Tag = Self;
}

impl Serialize<FieldIr> for &FieldIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FieldField::Id, &self.id)?;
        serializer.serialize::<tags::String, _>(FieldField::Name, &self.name)?;
        serializer.serialize::<tags::Bool, _>(FieldField::IsRequired, &self.is_required)?;
        serializer.serialize::<LexicalId, _>(FieldField::FieldType, &self.field_type)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct FieldIrBuilder {
    id: u32,
    name: String,
    doc: Option<String>,
    is_required: bool,
    field_type: LexicalId,
}

impl FieldIrBuilder {
    pub fn new(id: u32, name: impl Into<String>, is_required: bool, field_type: LexicalId) -> Self {
        Self {
            id,
            name: name.into(),
            doc: None,
            is_required,
            field_type,
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn finish(self) -> FieldIr {
        FieldIr {
            id: self.id,
            name: self.name,
            doc: self.doc,
            is_required: self.is_required,
            field_type: self.field_type,
        }
    }
}
