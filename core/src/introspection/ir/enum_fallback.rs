use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct EnumFallbackIr {
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
}

impl EnumFallbackIr {
    pub fn builder(name: impl Into<String>) -> EnumFallbackIrBuilder {
        EnumFallbackIrBuilder::new(name)
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
enum EnumFallbackField {
    Name = 0,
}

impl Tag for EnumFallbackIr {}

impl PrimaryTag for EnumFallbackIr {
    type Tag = Self;
}

impl Serialize<EnumFallbackIr> for &EnumFallbackIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String>(EnumFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct EnumFallbackIrBuilder {
    name: String,
    doc: Option<String>,
}

impl EnumFallbackIrBuilder {
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

    pub fn finish(self) -> EnumFallbackIr {
        EnumFallbackIr {
            name: self.name,
            doc: self.doc,
        }
    }
}
