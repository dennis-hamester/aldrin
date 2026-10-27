use super::LexicalId;
use crate::enum_repr::enum_with_repr;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};

#[derive(Debug, Copy, Clone)]
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

    #[expect(clippy::len_without_is_empty)]
    pub fn len(self) -> u32 {
        self.len
    }
}

enum_with_repr! {
    enum ArrayTypeField: u32 {
        ElemType = 0,
        Len = 1,
    }
}

impl Tag for ArrayTypeIr {}

impl PrimaryTag for ArrayTypeIr {
    type Tag = Self;
}

impl Serialize<ArrayTypeIr> for &ArrayTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId>(ArrayTypeField::ElemType, &self.elem_type)?;
        serializer.serialize::<tags::U32>(ArrayTypeField::Len, &self.len)?;

        serializer.finish()
    }
}
