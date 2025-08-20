use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) args: Option<LexicalId>,
    pub(crate) ok: Option<LexicalId>,
    pub(crate) err: Option<LexicalId>,
}

impl FunctionIr {
    pub(super) fn new(
        id: u32,
        name: impl Into<String>,
        args: Option<LexicalId>,
        ok: Option<LexicalId>,
        err: Option<LexicalId>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            args,
            ok,
            err,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> Option<LexicalId> {
        self.args
    }

    pub fn ok(&self) -> Option<LexicalId> {
        self.ok
    }

    pub fn err(&self) -> Option<LexicalId> {
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

impl Tag for FunctionIr {}

impl PrimaryTag for FunctionIr {
    type Tag = Self;
}

impl Serialize<Self> for FunctionIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<FunctionIr> for &FunctionIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FunctionField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(FunctionField::Name, &self.name)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Args, self.args)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Ok, self.ok)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Err, self.err)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for FunctionIr {
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
                    args = deserializer.deserialize::<tags::Option<LexicalId>, _>()?
                }

                Ok(FunctionField::Ok) => {
                    ok = deserializer.deserialize::<tags::Option<LexicalId>, _>()?
                }

                Ok(FunctionField::Err) => {
                    err = deserializer.deserialize::<tags::Option<LexicalId>, _>()?
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
