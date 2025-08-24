use super::{ir, resolve_ir, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Event {
    id: u32,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(rename = "type", default, skip_serializing_if = "Option::is_none")
    )]
    event_type: Option<TypeId>,
}

impl Event {
    pub fn from_ir(ev: ir::EventIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            id: ev.id,
            name: ev.name,
            doc: ev.doc,
            event_type: ev.event_type.map(|ty| resolve_ir(ty, references)),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn event_type(&self) -> Option<TypeId> {
        self.event_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EventField {
    Id = 0,
    Name = 1,
    Doc = 2,
    EventType = 3,
}

impl Tag for Event {}

impl PrimaryTag for Event {
    type Tag = Self;
}

impl Serialize<Self> for Event {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Event> for &Event {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32>(EventField::Id, &self.id)?;
        serializer.serialize::<tags::String>(EventField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(EventField::Doc, &self.doc)?;

        serializer
            .serialize_if_some::<tags::Option<TypeId>>(EventField::EventType, &self.event_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Event {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut doc = None;
        let mut event_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(EventField::Id) => id = deserializer.deserialize::<tags::U32, _>().map(Some)?,

                Ok(EventField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(EventField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(EventField::EventType) => {
                    event_type = deserializer.deserialize::<tags::Option<TypeId>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            event_type,
        })
    }
}
