use super::ir;
use crate::enum_repr::enum_with_repr;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct EventFallback {
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,
}

impl EventFallback {
    pub fn from_ir(fallback: ir::EventFallbackIr) -> Self {
        Self {
            name: fallback.name,
            doc: fallback.doc,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }
}

enum_with_repr! {
    enum EventFallbackField: u32 {
        Name = 0,
        Doc = 1,
    }
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

        serializer.serialize::<tags::String>(EventFallbackField::Name, &self.name)?;

        serializer
            .serialize_if_some::<tags::Option<tags::String>>(EventFallbackField::Doc, &self.doc)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for EventFallback {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut name = None;
        let mut doc = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EventFallbackField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(EventFallbackField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
        })
    }
}
