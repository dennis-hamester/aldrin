#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

impl PrimaryTag for String {
    type Tag = tags::String;
}

impl Serialize<tags::String> for String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<tags::String> for &String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

impl Deserialize<tags::String> for String {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_string()
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for String {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::String.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::STRING
    }

    fn add_references(_references: &mut References) {}
}

impl PrimaryTag for str {
    type Tag = tags::String;
}

impl Serialize<tags::String> for &str {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for str {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::String.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::STRING
    }

    fn add_references(_references: &mut References) {}
}
