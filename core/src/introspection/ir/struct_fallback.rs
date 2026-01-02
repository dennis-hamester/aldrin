use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct StructFallbackIr {
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
}

impl StructFallbackIr {
    pub fn builder(name: impl Into<String>) -> StructFallbackIrBuilder {
        StructFallbackIrBuilder::new(name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum StructFallbackField {
    Name = 0,
}

impl Tag for StructFallbackIr {}

impl PrimaryTag for StructFallbackIr {
    type Tag = Self;
}

impl Serialize<StructFallbackIr> for &StructFallbackIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(StructFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct StructFallbackIrBuilder {
    name: String,
    doc: Option<String>,
}

impl StructFallbackIrBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            doc: None,
        }
    }

    #[must_use]
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn finish(self) -> StructFallbackIr {
        StructFallbackIr {
            name: self.name,
            doc: self.doc,
        }
    }
}
