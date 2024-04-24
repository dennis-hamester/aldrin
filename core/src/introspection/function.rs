use super::TypeRef;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct Function {
    id: u32,
    name: String,
    args: Option<TypeRef>,
    ok: Option<TypeRef>,
    err: Option<TypeRef>,
}

impl Function {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> Option<&TypeRef> {
        self.args.as_ref()
    }

    pub fn ok(&self) -> Option<&TypeRef> {
        self.ok.as_ref()
    }

    pub fn err(&self) -> Option<&TypeRef> {
        self.err.as_ref()
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

        let id = deserializer.deserialize_specific_field(FunctionField::Id)?;
        let name = deserializer.deserialize_specific_field(FunctionField::Name)?;
        let args = deserializer.deserialize_specific_field(FunctionField::Args)?;
        let ok = deserializer.deserialize_specific_field(FunctionField::Ok)?;
        let err = deserializer.deserialize_specific_field(FunctionField::Err)?;

        deserializer.finish(Self {
            id,
            name,
            args,
            ok,
            err,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionBuilder {
    id: u32,
    name: String,
    args: Option<TypeRef>,
    ok: Option<TypeRef>,
    err: Option<TypeRef>,
}

impl FunctionBuilder {
    pub(crate) fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            args: None,
            ok: None,
            err: None,
        }
    }

    pub fn args(mut self, args: impl Into<TypeRef>) -> Self {
        self.args = Some(args.into());
        self
    }

    pub fn ok(mut self, ok: impl Into<TypeRef>) -> Self {
        self.ok = Some(ok.into());
        self
    }

    pub fn err(mut self, err: impl Into<TypeRef>) -> Self {
        self.err = Some(err.into());
        self
    }

    pub fn finish(self) -> Function {
        Function {
            id: self.id,
            name: self.name,
            args: self.args,
            ok: self.ok,
            err: self.err,
        }
    }
}
