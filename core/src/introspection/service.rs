use super::{ir, Event, Function, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer,
    ServiceUuid, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Service {
    schema: String,
    name: String,
    uuid: ServiceUuid,
    version: u32,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "HashMap::is_empty")
    )]
    functions: HashMap<u32, Function>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "HashMap::is_empty")
    )]
    events: HashMap<u32, Event>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    function_fallback: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    event_fallback: Option<String>,
}

impl Service {
    pub fn from_ir(ty: ir::ServiceIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            schema: ty.schema,
            name: ty.name,
            uuid: ty.uuid,
            version: ty.version,

            functions: ty
                .functions
                .into_iter()
                .map(|(id, func)| (id, Function::from_ir(func, references)))
                .collect(),

            events: ty
                .events
                .into_iter()
                .map(|(id, ev)| (id, Event::from_ir(ev, references)))
                .collect(),

            function_fallback: ty.function_fallback,
            event_fallback: ty.event_fallback,
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uuid(&self) -> ServiceUuid {
        self.uuid
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn functions(&self) -> &HashMap<u32, Function> {
        &self.functions
    }

    pub fn events(&self) -> &HashMap<u32, Event> {
        &self.events
    }

    pub fn function_fallback(&self) -> Option<&str> {
        self.function_fallback.as_deref()
    }

    pub fn event_fallback(&self) -> Option<&str> {
        self.event_fallback.as_deref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceField {
    Schema = 0,
    Name = 1,
    Uuid = 2,
    Version = 3,
    Functions = 4,
    Events = 5,
    FunctionFallback = 6,
    EventFallback = 7,
}

impl Tag for Service {}

impl PrimaryTag for Service {
    type Tag = Self;
}

impl Serialize<Self> for Service {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Service> for &Service {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(ServiceField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(ServiceField::Name, &self.name)?;
        serializer.serialize::<ServiceUuid, _>(ServiceField::Uuid, &self.uuid)?;
        serializer.serialize::<tags::U32, _>(ServiceField::Version, &self.version)?;

        serializer.serialize::<tags::Map<tags::U32, Function>, _>(
            ServiceField::Functions,
            &self.functions,
        )?;

        serializer
            .serialize::<tags::Map<tags::U32, Event>, _>(ServiceField::Events, &self.events)?;

        serializer.serialize_if_some::<tags::Option<tags::String>, _>(
            ServiceField::FunctionFallback,
            &self.function_fallback,
        )?;

        serializer.serialize_if_some::<tags::Option<tags::String>, _>(
            ServiceField::EventFallback,
            &self.event_fallback,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Service {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut uuid = None;
        let mut version = None;
        let mut functions = None;
        let mut events = None;
        let mut function_fallback = None;
        let mut event_fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ServiceField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(ServiceField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(ServiceField::Uuid) => {
                    uuid = deserializer.deserialize::<ServiceUuid, _>().map(Some)?
                }

                Ok(ServiceField::Version) => {
                    version = deserializer.deserialize::<tags::U32, _>().map(Some)?
                }

                Ok(ServiceField::Functions) => {
                    functions = deserializer
                        .deserialize::<tags::Map<tags::U32, Function>, _>()
                        .map(Some)?
                }

                Ok(ServiceField::Events) => {
                    events = deserializer
                        .deserialize::<tags::Map<tags::U32, Event>, _>()
                        .map(Some)?
                }

                Ok(ServiceField::FunctionFallback) => {
                    function_fallback =
                        deserializer.deserialize::<tags::Option<tags::String>, _>()?
                }

                Ok(ServiceField::EventFallback) => {
                    event_fallback = deserializer.deserialize::<tags::Option<tags::String>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            uuid: uuid.ok_or(DeserializeError::InvalidSerialization)?,
            version: version.ok_or(DeserializeError::InvalidSerialization)?,
            functions: functions.ok_or(DeserializeError::InvalidSerialization)?,
            events: events.ok_or(DeserializeError::InvalidSerialization)?,
            function_fallback,
            event_fallback,
        })
    }
}
