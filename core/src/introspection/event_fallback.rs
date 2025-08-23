use super::ir;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct EventFallback {
    name: String,
}

impl EventFallback {
    pub fn from_ir(func: ir::EventFallbackIr) -> Self {
        Self { name: func.name }
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

impl Tag for EventFallback {}

impl PrimaryTag for EventFallback {
    type Tag = Self;
}

impl Serialize<Self> for EventFallback {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<EventFallback> for &EventFallback {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(EventFallbackField::Name, &self.name)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for EventFallback {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut name = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EventFallbackField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
