use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArrayType {
    elem_type: LexicalId,
    len: u32,
}

impl ArrayType {
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

impl Serialize for ArrayType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ArrayTypeField::ElemType, &self.elem_type)?;
        serializer.serialize_field(ArrayTypeField::Len, &self.len)?;

        serializer.finish()
    }
}

impl Deserialize for ArrayType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let elem_type = deserializer.deserialize_specific_field(ArrayTypeField::ElemType)?;
        let len = deserializer.deserialize_specific_field(ArrayTypeField::Len)?;

        deserializer.finish(Self { elem_type, len })
    }
}
