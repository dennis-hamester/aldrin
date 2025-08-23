use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
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

impl Serialize<FunctionIr> for &FunctionIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FunctionField::Id, &self.id)?;
        serializer.serialize::<tags::String, _>(FunctionField::Name, &self.name)?;

        serializer
            .serialize_if_some::<tags::Option<LexicalId>, _>(FunctionField::Args, &self.args)?;

        serializer.serialize_if_some::<tags::Option<LexicalId>, _>(FunctionField::Ok, &self.ok)?;

        serializer
            .serialize_if_some::<tags::Option<LexicalId>, _>(FunctionField::Err, &self.err)?;

        serializer.finish()
    }
}
