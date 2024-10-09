use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResultType {
    ok: LexicalId,
    err: LexicalId,
}

impl ResultType {
    pub fn new(ok: LexicalId, err: LexicalId) -> Self {
        Self { ok, err }
    }

    pub fn ok(self) -> LexicalId {
        self.ok
    }

    pub fn err(self) -> LexicalId {
        self.err
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultTypeField {
    Ok = 0,
    Err = 1,
}

impl Serialize for ResultType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ResultTypeField::Ok, &self.ok)?;
        serializer.serialize_field(ResultTypeField::Err, &self.err)?;

        serializer.finish()
    }
}

impl Deserialize for ResultType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let ok = deserializer.deserialize_specific_field(ResultTypeField::Ok)?;
        let err = deserializer.deserialize_specific_field(ResultTypeField::Err)?;

        deserializer.finish(Self { ok, err })
    }
}
