use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct EventFallbackIr {
    pub(crate) name: String,
}

impl EventFallbackIr {
    pub fn builder(name: impl Into<String>) -> EventFallbackIrBuilder {
        EventFallbackIrBuilder::new(name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EventFallbackField {
    Name = 0,
}

impl Tag for EventFallbackIr {}

impl PrimaryTag for EventFallbackIr {
    type Tag = Self;
}

impl Serialize<EventFallbackIr> for &EventFallbackIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(EventFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct EventFallbackIrBuilder {
    name: String,
}

impl EventFallbackIrBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn finish(self) -> EventFallbackIr {
        EventFallbackIr { name: self.name }
    }
}
