use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Function {
    id: u32,
    name: String,
    args: Option<LexicalId>,
    ok: Option<LexicalId>,
    err: Option<LexicalId>,
}

impl Function {
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
        let mut serializer = serializer.serialize_struct1(5)?;

        serializer.serialize::<tags::U32, _>(FunctionField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(FunctionField::Name, &self.name)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Args, self.args)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Ok, self.ok)?;
        serializer.serialize::<tags::Option<LexicalId>, _>(FunctionField::Err, self.err)?;

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
