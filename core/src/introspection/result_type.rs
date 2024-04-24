use super::{BuiltInType, TypeRef};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    ok: TypeRef,
    err: TypeRef,
}

impl ResultType {
    pub fn new(ok: impl Into<TypeRef>, err: impl Into<TypeRef>) -> Self {
        Self {
            ok: ok.into(),
            err: err.into(),
        }
    }

    pub fn ok(&self) -> &TypeRef {
        &self.ok
    }

    pub fn err(&self) -> &TypeRef {
        &self.err
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

impl From<ResultType> for BuiltInType {
    fn from(t: ResultType) -> Self {
        BuiltInType::Result(Box::new(t))
    }
}

impl From<ResultType> for TypeRef {
    fn from(t: ResultType) -> Self {
        Self::BuiltIn(t.into())
    }
}
