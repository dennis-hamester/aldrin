use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayTypeIr {
    pub(crate) elem_type: LexicalId,
    pub(crate) len: u32,
}

impl ArrayTypeIr {
    pub fn new(elem_type: LexicalId, len: u32) -> Self {
        Self { elem_type, len }
    }

    pub fn elem_type(self) -> LexicalId {
        self.elem_type
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> u32 {
        self.len
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ArrayTypeField {
    ElemType = 0,
    Len = 1,
}

impl Tag for ArrayTypeIr {}

impl PrimaryTag for ArrayTypeIr {
    type Tag = Self;
}

impl Serialize<ArrayTypeIr> for &ArrayTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId, _>(ArrayTypeField::ElemType, &self.elem_type)?;
        serializer.serialize::<tags::U32, _>(ArrayTypeField::Len, &self.len)?;

        serializer.finish()
    }
}
