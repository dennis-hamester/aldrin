#[cfg(feature = "introspection")]
use crate::introspection::{Introspectable, LexicalId, References, ir};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

impl PrimaryTag for () {
    type Tag = tags::Unit;
}

impl Serialize<tags::Unit> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_none()
    }
}

impl Serialize<tags::Unit> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_none()
    }
}

impl Deserialize<tags::Unit> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_none()
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for () {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Unit.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UNIT
    }

    fn add_references(_references: &mut References) {}
}
