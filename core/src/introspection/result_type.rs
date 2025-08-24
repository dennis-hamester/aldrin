use super::{ir, resolve_ir, LexicalId};
use crate::tags::{PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct ResultType {
    ok: TypeId,
    err: TypeId,
}

impl ResultType {
    pub fn from_ir(ty: ir::ResultTypeIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            ok: resolve_ir(ty.ok, references),
            err: resolve_ir(ty.err, references),
        }
    }

    pub fn ok(self) -> TypeId {
        self.ok
    }

    pub fn err(self) -> TypeId {
        self.err
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultTypeField {
    Ok = 0,
    Err = 1,
}

impl Tag for ResultType {}

impl PrimaryTag for ResultType {
    type Tag = Self;
}

impl Serialize<Self> for ResultType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<TypeId>(ResultTypeField::Ok, self.ok)?;
        serializer.serialize::<TypeId>(ResultTypeField::Err, self.err)?;

        serializer.finish()
    }
}

impl Serialize<ResultType> for &ResultType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ResultType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut ok = None;
        let mut err = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ResultTypeField::Ok) => {
                    ok = deserializer.deserialize::<TypeId, _>().map(Some)?;
                }

                Ok(ResultTypeField::Err) => {
                    err = deserializer.deserialize::<TypeId, _>().map(Some)?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            ok: ok.ok_or(DeserializeError::InvalidSerialization)?,
            err: err.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
