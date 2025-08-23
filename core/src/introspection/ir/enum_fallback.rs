use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct EnumFallbackIr {
    pub(crate) name: String,
}

impl EnumFallbackIr {
    pub fn builder(name: impl Into<String>) -> EnumFallbackIrBuilder {
        EnumFallbackIrBuilder::new(name)
    }

    pub fn name(&self) -> &str {
        &self.name
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

        serializer.serialize::<tags::String, _>(EnumFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct EnumFallbackIrBuilder {
    name: String,
}

impl EnumFallbackIrBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn finish(self) -> EnumFallbackIr {
        EnumFallbackIr { name: self.name }
    }
}
