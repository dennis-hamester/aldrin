use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl Serialize<Self> for ArrayTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId, _>(ArrayTypeField::ElemType, self.elem_type)?;
        serializer.serialize::<tags::U32, _>(ArrayTypeField::Len, self.len)?;

        serializer.finish()
    }
}

impl Serialize<ArrayTypeIr> for &ArrayTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ArrayTypeIr {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut elem_type = None;
        let mut len = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ArrayTypeField::ElemType) => {
                    elem_type = deserializer.deserialize::<LexicalId, _>().map(Some)?;
                }

                Ok(ArrayTypeField::Len) => {
                    len = deserializer.deserialize::<tags::U32, _>().map(Some)?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            elem_type: elem_type.ok_or(DeserializeError::InvalidSerialization)?,
            len: len.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
