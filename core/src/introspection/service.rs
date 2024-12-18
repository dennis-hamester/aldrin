use super::{Event, Function, LexicalId};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceUuid;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
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
}

impl Serialize for Service {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(6)?;

        serializer.serialize_field(ServiceField::Schema, &self.schema)?;
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

        let schema = deserializer.deserialize_specific_field(ServiceField::Schema)?;
        let name = deserializer.deserialize_specific_field(ServiceField::Name)?;
        let uuid = deserializer.deserialize_specific_field(ServiceField::Uuid)?;
        let version = deserializer.deserialize_specific_field(ServiceField::Version)?;
        let functions = deserializer.deserialize_specific_field(ServiceField::Functions)?;
        let events = deserializer.deserialize_specific_field(ServiceField::Events)?;

        deserializer.finish(Self {
            schema,
            name,
            uuid,
            version,
            functions,
            events,
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

    pub fn finish(self) -> Service {
        Service {
            schema: self.schema,
            name: self.name,
            uuid: self.uuid,
            version: self.version,
            functions: self.functions,
            events: self.events,
        }
    }
}
