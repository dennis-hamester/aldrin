use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct FunctionIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
    pub(crate) args: Option<LexicalId>,
    pub(crate) ok: Option<LexicalId>,
    pub(crate) err: Option<LexicalId>,
}

impl FunctionIr {
    pub fn builder(id: u32, name: impl Into<String>) -> FunctionIrBuilder {
        FunctionIrBuilder::new(id, name)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
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

        serializer.serialize::<tags::U32>(FunctionField::Id, &self.id)?;
        serializer.serialize::<tags::String>(FunctionField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<LexicalId>>(FunctionField::Args, &self.args)?;
        serializer.serialize_if_some::<tags::Option<LexicalId>>(FunctionField::Ok, &self.ok)?;
        serializer.serialize_if_some::<tags::Option<LexicalId>>(FunctionField::Err, &self.err)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionIrBuilder {
    id: u32,
    name: String,
    doc: Option<String>,
    args: Option<LexicalId>,
    ok: Option<LexicalId>,
    err: Option<LexicalId>,
}

impl FunctionIrBuilder {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            doc: None,
            args: None,
            ok: None,
            err: None,
        }
    }

    #[must_use]
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    #[must_use]
    pub fn args(mut self, args: LexicalId) -> Self {
        self.args = Some(args);
        self
    }

    #[must_use]
    pub fn ok(mut self, ok: LexicalId) -> Self {
        self.ok = Some(ok);
        self
    }

    #[must_use]
    pub fn err(mut self, err: LexicalId) -> Self {
        self.err = Some(err);
        self
    }

    pub fn finish(self) -> FunctionIr {
        FunctionIr {
            id: self.id,
            name: self.name,
            doc: self.doc,
            args: self.args,
            ok: self.ok,
            err: self.err,
        }
    }
}
