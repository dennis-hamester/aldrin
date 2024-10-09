use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct LexicalId(pub Uuid);

impl LexicalId {
    pub const NIL: Self = Self(Uuid::nil());

    pub const fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl Serialize for LexicalId {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0);
        Ok(())
    }
}

impl Deserialize for LexicalId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}
