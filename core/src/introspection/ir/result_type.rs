use super::LexicalId;
use crate::tags::{PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResultTypeIr {
    pub(crate) ok: LexicalId,
    pub(crate) err: LexicalId,
}

impl ResultTypeIr {
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

impl Tag for ResultTypeIr {}

impl PrimaryTag for ResultTypeIr {
    type Tag = Self;
}

impl Serialize<ResultTypeIr> for &ResultTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId, _>(ResultTypeField::Ok, &self.ok)?;
        serializer.serialize::<LexicalId, _>(ResultTypeField::Err, &self.err)?;

        serializer.finish()
    }
}
