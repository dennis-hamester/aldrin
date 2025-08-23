use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct EventIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) event_type: Option<LexicalId>,
}

impl EventIr {
    pub fn builder(id: u32, name: impl Into<String>) -> EventIrBuilder {
        EventIrBuilder::new(id, name)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn event_type(&self) -> Option<LexicalId> {
        self.event_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EventField {
    Id = 0,
    Name = 1,
    EventType = 2,
}

impl Tag for EventIr {}

impl PrimaryTag for EventIr {
    type Tag = Self;
}

impl Serialize<EventIr> for &EventIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(EventField::Id, &self.id)?;
        serializer.serialize::<tags::String, _>(EventField::Name, &self.name)?;

        serializer.serialize_if_some::<tags::Option<LexicalId>, _>(
            EventField::EventType,
            &self.event_type,
        )?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct EventIrBuilder {
    id: u32,
    name: String,
    event_type: Option<LexicalId>,
}

impl EventIrBuilder {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            event_type: None,
        }
    }

    pub fn event_type(mut self, event_type: LexicalId) -> Self {
        self.event_type = Some(event_type);
        self
    }

    pub fn finish(self) -> EventIr {
        EventIr {
            id: self.id,
            name: self.name,
            event_type: self.event_type,
        }
    }
}
