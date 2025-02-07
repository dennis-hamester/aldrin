use super::{Event, Function, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, ServiceUuid,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Service {
    schema: String,
    name: String,
    uuid: ServiceUuid,
    version: u32,
    functions: BTreeMap<u32, Function>,
    events: BTreeMap<u32, Event>,
    function_fallback: Option<String>,
    event_fallback: Option<String>,
}

impl Service {
    pub const NAMESPACE: Uuid = uuid!("de06b048-55f7-43b9-8d34-555795c2f4c6");

    pub fn builder(
        schema: impl Into<String>,
        name: impl Into<String>,
        uuid: ServiceUuid,
        version: u32,
    ) -> ServiceBuilder {
        ServiceBuilder::new(schema, name, uuid, version)
    }

    pub fn lexical_id(&self) -> LexicalId {
        LexicalId::service(&self.schema, &self.name)
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

    pub fn functions(&self) -> &BTreeMap<u32, Function> {
        &self.functions
    }

    pub fn events(&self) -> &BTreeMap<u32, Event> {
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
        let num = 6
            + (self.function_fallback.is_some() as usize)
            + (self.event_fallback.is_some() as usize);
        let mut serializer = serializer.serialize_struct(num)?;

        serializer.serialize::<tags::String, _>(ServiceField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(ServiceField::Name, &self.name)?;
        serializer.serialize::<ServiceUuid, _>(ServiceField::Uuid, self.uuid)?;
        serializer.serialize::<tags::U32, _>(ServiceField::Version, self.version)?;

        serializer.serialize::<tags::Map<tags::U32, Function>, _>(
            ServiceField::Functions,
            &self.functions,
        )?;

        serializer
            .serialize::<tags::Map<tags::U32, Event>, _>(ServiceField::Events, &self.events)?;

        if self.function_fallback.is_some() {
            serializer.serialize::<tags::Option<tags::String>, _>(
                ServiceField::FunctionFallback,
                &self.function_fallback,
            )?;
        }

        if self.event_fallback.is_some() {
            serializer.serialize::<tags::Option<tags::String>, _>(
                ServiceField::EventFallback,
                &self.event_fallback,
            )?;
        }

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

        while !deserializer.is_empty() {
            let deserializer = deserializer.deserialize()?;

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

#[derive(Debug, Clone)]
pub struct ServiceBuilder {
    schema: String,
    name: String,
    uuid: ServiceUuid,
    version: u32,
    functions: BTreeMap<u32, Function>,
    events: BTreeMap<u32, Event>,
    function_fallback: Option<String>,
    event_fallback: Option<String>,
}

impl ServiceBuilder {
    pub fn new(
        schema: impl Into<String>,
        name: impl Into<String>,
        uuid: ServiceUuid,
        version: u32,
    ) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            uuid,
            version,
            functions: BTreeMap::new(),
            events: BTreeMap::new(),
            function_fallback: None,
            event_fallback: None,
        }
    }

    pub fn function(
        mut self,
        id: u32,
        name: impl Into<String>,
        args: Option<LexicalId>,
        ok: Option<LexicalId>,
        err: Option<LexicalId>,
    ) -> Self {
        self.functions
            .insert(id, Function::new(id, name, args, ok, err));
        self
    }

    pub fn event(
        mut self,
        id: u32,
        name: impl Into<String>,
        event_type: Option<LexicalId>,
    ) -> Self {
        self.events.insert(id, Event::new(id, name, event_type));
        self
    }

    pub fn function_fallback(mut self, name: impl Into<String>) -> Self {
        self.function_fallback = Some(name.into());
        self
    }

    pub fn event_fallback(mut self, name: impl Into<String>) -> Self {
        self.event_fallback = Some(name.into());
        self
    }

    pub fn finish(self) -> Service {
        Service {
            schema: self.schema,
            name: self.name,
            uuid: self.uuid,
            version: self.version,
            functions: self.functions,
            events: self.events,
            function_fallback: self.function_fallback,
            event_fallback: self.event_fallback,
        }
    }
}
