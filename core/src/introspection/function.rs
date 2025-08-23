use super::{ir, resolve_ir, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Function {
    id: u32,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    args: Option<TypeId>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    ok: Option<TypeId>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    err: Option<TypeId>,
}

impl Function {
    pub fn from_ir(func: ir::FunctionIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            id: func.id,
            name: func.name,
            args: func.args.map(|ty| resolve_ir(ty, references)),
            ok: func.ok.map(|ty| resolve_ir(ty, references)),
            err: func.err.map(|ty| resolve_ir(ty, references)),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> Option<TypeId> {
        self.args
    }

    pub fn ok(&self) -> Option<TypeId> {
        self.ok
    }

    pub fn err(&self) -> Option<TypeId> {
        self.err
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FunctionField {
    Id = 0,
    Name = 1,
    Args = 2,
    Ok = 3,
    Err = 4,
}

impl Tag for Function {}

impl PrimaryTag for Function {
    type Tag = Self;
}

impl Serialize<Self> for Function {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Function> for &Function {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FunctionField::Id, &self.id)?;
        serializer.serialize::<tags::String, _>(FunctionField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<TypeId>, _>(FunctionField::Args, &self.args)?;
        serializer.serialize_if_some::<tags::Option<TypeId>, _>(FunctionField::Ok, &self.ok)?;
        serializer.serialize_if_some::<tags::Option<TypeId>, _>(FunctionField::Err, &self.err)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Function {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut args = None;
        let mut ok = None;
        let mut err = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(FunctionField::Id) => {
                    id = deserializer.deserialize::<tags::U32, _>().map(Some)?
                }

                Ok(FunctionField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(FunctionField::Args) => {
                    args = deserializer.deserialize::<tags::Option<TypeId>, _>()?
                }

                Ok(FunctionField::Ok) => {
                    ok = deserializer.deserialize::<tags::Option<TypeId>, _>()?
                }

                Ok(FunctionField::Err) => {
                    err = deserializer.deserialize::<tags::Option<TypeId>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            args,
            ok,
            err,
        })
    }
}
