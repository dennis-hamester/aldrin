use super::{Event, EventBuilder, Function, FunctionBuilder, Layout};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceUuid;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct Service {
    name: String,
    uuid: ServiceUuid,
    version: u32,
    functions: BTreeMap<u32, Function>,
    events: BTreeMap<u32, Event>,
}

impl Service {
    pub const NAMESPACE: Uuid = uuid!("de06b048-55f7-43b9-8d34-555795c2f4c6");

    pub fn builder(name: impl Into<String>, uuid: ServiceUuid, version: u32) -> ServiceBuilder {
        ServiceBuilder::new(name, uuid, version)
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
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceField {
    Name = 0,
    Uuid = 1,
    Version = 2,
    Functions = 3,
    Events = 4,
}

impl Serialize for Service {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(5)?;

        serializer.serialize_field(ServiceField::Name, &self.name)?;
        serializer.serialize_field(ServiceField::Uuid, &self.uuid)?;
        serializer.serialize_field(ServiceField::Version, &self.version)?;
        serializer.serialize_field(ServiceField::Functions, &self.functions)?;
        serializer.serialize_field(ServiceField::Events, &self.events)?;

        serializer.finish()
    }
}

impl Deserialize for Service {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let name = deserializer.deserialize_specific_field(ServiceField::Name)?;
        let uuid = deserializer.deserialize_specific_field(ServiceField::Uuid)?;
        let version = deserializer.deserialize_specific_field(ServiceField::Version)?;
        let functions = deserializer.deserialize_specific_field(ServiceField::Functions)?;
        let events = deserializer.deserialize_specific_field(ServiceField::Events)?;

        deserializer.finish(Self {
            name,
            uuid,
            version,
            functions,
            events,
        })
    }
}

impl From<Service> for Layout {
    fn from(s: Service) -> Self {
        Self::Service(s)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceBuilder {
    name: String,
    uuid: ServiceUuid,
    version: u32,
    functions: BTreeMap<u32, Function>,
    events: BTreeMap<u32, Event>,
}

impl ServiceBuilder {
    pub fn new(name: impl Into<String>, uuid: ServiceUuid, version: u32) -> Self {
        Self {
            name: name.into(),
            uuid,
            version,
            functions: BTreeMap::new(),
            events: BTreeMap::new(),
        }
    }

    pub fn function(
        mut self,
        id: u32,
        name: impl Into<String>,
        f: impl FnOnce(FunctionBuilder) -> Function,
    ) -> Self {
        self.functions.insert(id, f(FunctionBuilder::new(id, name)));
        self
    }

    pub fn event(
        mut self,
        id: u32,
        name: impl Into<String>,
        f: impl FnOnce(EventBuilder) -> Event,
    ) -> Self {
        self.events.insert(id, f(EventBuilder::new(id, name)));
        self
    }

    pub fn finish(self) -> Service {
        Service {
            name: self.name,
            uuid: self.uuid,
            version: self.version,
            functions: self.functions,
            events: self.events,
        }
    }
}
