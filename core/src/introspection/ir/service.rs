use super::{EventIr, FunctionIr, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer, ServiceUuid};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct ServiceIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) uuid: ServiceUuid,
    pub(crate) version: u32,
    pub(crate) functions: BTreeMap<u32, FunctionIr>,
    pub(crate) events: BTreeMap<u32, EventIr>,
    pub(crate) function_fallback: Option<String>,
    pub(crate) event_fallback: Option<String>,
}

impl ServiceIr {
    pub const NAMESPACE: Uuid = uuid!("de06b048-55f7-43b9-8d34-555795c2f4c6");

    pub fn builder(
        schema: impl Into<String>,
        name: impl Into<String>,
        uuid: ServiceUuid,
        version: u32,
    ) -> ServiceIrBuilder {
        ServiceIrBuilder::new(schema, name, uuid, version)
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

    pub fn functions(&self) -> &BTreeMap<u32, FunctionIr> {
        &self.functions
    }

    pub fn events(&self) -> &BTreeMap<u32, EventIr> {
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

impl Tag for ServiceIr {}

impl PrimaryTag for ServiceIr {
    type Tag = Self;
}

impl Serialize<ServiceIr> for &ServiceIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(ServiceField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(ServiceField::Name, &self.name)?;
        serializer.serialize::<ServiceUuid, _>(ServiceField::Uuid, &self.uuid)?;
        serializer.serialize::<tags::U32, _>(ServiceField::Version, &self.version)?;

        serializer.serialize::<tags::Map<tags::U32, FunctionIr>, _>(
            ServiceField::Functions,
            &self.functions,
        )?;

        serializer
            .serialize::<tags::Map<tags::U32, EventIr>, _>(ServiceField::Events, &self.events)?;

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

#[derive(Debug, Clone)]
pub struct ServiceIrBuilder {
    schema: String,
    name: String,
    uuid: ServiceUuid,
    version: u32,
    functions: BTreeMap<u32, FunctionIr>,
    events: BTreeMap<u32, EventIr>,
    function_fallback: Option<String>,
    event_fallback: Option<String>,
}

impl ServiceIrBuilder {
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
            .insert(id, FunctionIr::new(id, name, args, ok, err));
        self
    }

    pub fn event(
        mut self,
        id: u32,
        name: impl Into<String>,
        event_type: Option<LexicalId>,
    ) -> Self {
        self.events.insert(id, EventIr::new(id, name, event_type));
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

    pub fn finish(self) -> ServiceIr {
        ServiceIr {
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
