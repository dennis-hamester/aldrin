use super::{ir, Event, EventFallback, Function, FunctionFallback, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer,
    ServiceUuid, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Service {
    schema: String,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,
    uuid: ServiceUuid,
    version: u32,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    functions: BTreeMap<u32, Function>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    events: BTreeMap<u32, Event>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    function_fallback: Option<FunctionFallback>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    event_fallback: Option<EventFallback>,
}

impl Service {
    pub fn from_ir(ty: ir::ServiceIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            schema: ty.schema,
            name: ty.name,
            doc: ty.doc,
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

            function_fallback: ty.function_fallback.map(FunctionFallback::from_ir),
            event_fallback: ty.event_fallback.map(EventFallback::from_ir),
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn uuid(&self) -> ServiceUuid {
        self.uuid
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn functions(&self) -> &BTreeMap<u32, Function> {
        &self.functions
    }

    pub fn events(&self) -> &BTreeMap<u32, Event> {
        &self.events
    }

    pub fn function_fallback(&self) -> Option<&FunctionFallback> {
        self.function_fallback.as_ref()
    }

    pub fn event_fallback(&self) -> Option<&EventFallback> {
        self.event_fallback.as_ref()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceField {
    Schema = 0,
    Name = 1,
    Doc = 2,
    Uuid = 3,
    Version = 4,
    Functions = 5,
    Events = 6,
    FunctionFallback = 7,
    EventFallback = 8,
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

        serializer.serialize::<tags::String>(ServiceField::Schema, &self.schema)?;
        serializer.serialize::<tags::String>(ServiceField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(ServiceField::Doc, &self.doc)?;
        serializer.serialize::<ServiceUuid>(ServiceField::Uuid, &self.uuid)?;
        serializer.serialize::<tags::U32>(ServiceField::Version, &self.version)?;

        serializer.serialize::<tags::Map<tags::U32, Function>>(
            ServiceField::Functions,
            &self.functions,
        )?;

        serializer.serialize::<tags::Map<tags::U32, Event>>(ServiceField::Events, &self.events)?;

        serializer.serialize_if_some::<tags::Option<FunctionFallback>>(
            ServiceField::FunctionFallback,
            &self.function_fallback,
        )?;

        serializer.serialize_if_some::<tags::Option<EventFallback>>(
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
        let mut doc = None;
        let mut uuid = None;
        let mut version = None;
        let mut functions = None;
        let mut events = None;
        let mut function_fallback = None;
        let mut event_fallback = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ServiceField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(ServiceField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(ServiceField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(ServiceField::Uuid) => {
                    uuid = deserializer.deserialize::<ServiceUuid, _>().map(Some)?;
                }

                Ok(ServiceField::Version) => {
                    version = deserializer.deserialize::<tags::U32, _>().map(Some)?;
                }

                Ok(ServiceField::Functions) => {
                    functions = deserializer
                        .deserialize::<tags::Map<tags::U32, Function>, _>()
                        .map(Some)?;
                }

                Ok(ServiceField::Events) => {
                    events = deserializer
                        .deserialize::<tags::Map<tags::U32, Event>, _>()
                        .map(Some)?;
                }

                Ok(ServiceField::FunctionFallback) => {
                    function_fallback =
                        deserializer.deserialize::<tags::Option<FunctionFallback>, _>()?;
                }

                Ok(ServiceField::EventFallback) => {
                    event_fallback =
                        deserializer.deserialize::<tags::Option<EventFallback>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            uuid: uuid.ok_or(DeserializeError::InvalidSerialization)?,
            version: version.ok_or(DeserializeError::InvalidSerialization)?,
            functions: functions.ok_or(DeserializeError::InvalidSerialization)?,
            events: events.ok_or(DeserializeError::InvalidSerialization)?,
            function_fallback,
            event_fallback,
        })
    }
}
