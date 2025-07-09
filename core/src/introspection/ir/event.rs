use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) event_type: Option<LexicalId>,
}

impl EventIr {
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

impl Tag for EventIr {}

impl PrimaryTag for EventIr {
    type Tag = Self;
}

impl Serialize<Self> for EventIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<EventIr> for &EventIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct1(3)?;

        serializer.serialize::<tags::U32, _>(EventField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(EventField::Name, &self.name)?;

        serializer
            .serialize::<tags::Option<LexicalId>, _>(EventField::EventType, self.event_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for EventIr {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut event_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EventField::Id) => id = deserializer.deserialize::<tags::U32, _>().map(Some)?,

                Ok(EventField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(EventField::EventType) => {
                    event_type = deserializer.deserialize::<tags::Option<LexicalId>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            event_type,
        })
    }
}
