use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Event {
    id: u32,
    name: String,
    event_type: Option<LexicalId>,
}

impl Event {
    pub(super) fn new(id: u32, name: impl Into<String>, event_type: Option<LexicalId>) -> Self {
        Self {
            id,
            name: name.into(),
            event_type,
        }
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

impl Serialize for Event {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(EventField::Id, &self.id)?;
        serializer.serialize_field(EventField::Name, &self.name)?;
        serializer.serialize_field(EventField::EventType, &self.event_type)?;

        serializer.finish()
    }
}

impl Deserialize for Event {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_specific_field(EventField::Id)?;
        let name = deserializer.deserialize_specific_field(EventField::Name)?;
        let event_type = deserializer.deserialize_specific_field(EventField::EventType)?;

        deserializer.finish(Self {
            id,
            name,
            event_type,
        })
    }
}
