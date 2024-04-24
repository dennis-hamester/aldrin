use super::TypeRef;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct Event {
    id: u32,
    name: String,
    data: Option<TypeRef>,
}

impl Event {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> Option<&TypeRef> {
        self.data.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EventField {
    Id = 0,
    Name = 1,
    Data = 2,
}

impl Serialize for Event {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(EventField::Id, &self.id)?;
        serializer.serialize_field(EventField::Name, &self.name)?;
        serializer.serialize_field(EventField::Data, &self.data)?;

        serializer.finish()
    }
}

impl Deserialize for Event {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_specific_field(EventField::Id)?;
        let name = deserializer.deserialize_specific_field(EventField::Name)?;
        let data = deserializer.deserialize_specific_field(EventField::Data)?;

        deserializer.finish(Self { id, name, data })
    }
}

#[derive(Debug, Clone)]
pub struct EventBuilder {
    id: u32,
    name: String,
    data: Option<TypeRef>,
}

impl EventBuilder {
    pub(crate) fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            data: None,
        }
    }

    pub fn data(mut self, data: impl Into<TypeRef>) -> Self {
        self.data = Some(data.into());
        self
    }

    pub fn finish(self) -> Event {
        Event {
            id: self.id,
            name: self.name,
            data: self.data,
        }
    }
}
