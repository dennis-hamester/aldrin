use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct FunctionFallbackIr {
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
}

impl FunctionFallbackIr {
    pub fn builder(name: impl Into<String>) -> FunctionFallbackIrBuilder {
        FunctionFallbackIrBuilder::new(name)
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
enum FunctionFallbackField {
    Name = 0,
}

impl Tag for FunctionFallbackIr {}

impl PrimaryTag for FunctionFallbackIr {
    type Tag = Self;
}

impl Serialize<FunctionFallbackIr> for &FunctionFallbackIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(FunctionFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionFallbackIrBuilder {
    name: String,
    doc: Option<String>,
}

impl FunctionFallbackIrBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            doc: None,
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn finish(self) -> FunctionFallbackIr {
        FunctionFallbackIr {
            name: self.name,
            doc: self.doc,
        }
    }
}
