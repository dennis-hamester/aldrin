use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
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

impl Serialize for Function {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(5)?;

        serializer.serialize_field(FunctionField::Id, &self.id)?;
        serializer.serialize_field(FunctionField::Name, &self.name)?;
        serializer.serialize_field(FunctionField::Args, &self.args)?;
        serializer.serialize_field(FunctionField::Ok, &self.ok)?;
        serializer.serialize_field(FunctionField::Err, &self.err)?;

        serializer.finish()
    }
}

impl Deserialize for Function {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut args = None;
        let mut ok = None;
        let mut err = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id()? {
                FunctionField::Id => id = deserializer.deserialize().map(Some)?,
                FunctionField::Name => name = deserializer.deserialize().map(Some)?,
                FunctionField::Args => args = deserializer.deserialize()?,
                FunctionField::Ok => ok = deserializer.deserialize()?,
                FunctionField::Err => err = deserializer.deserialize()?,
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
