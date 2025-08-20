use super::{ir, resolve_ir, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct ArrayType {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    elem_type: TypeId,

    len: u32,
}

impl ArrayType {
    pub fn from_ir(ty: ir::ArrayTypeIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            elem_type: resolve_ir(ty.elem_type, references),
            len: ty.len,
        }
    }

    pub fn elem_type(self) -> TypeId {
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

impl Tag for ArrayType {}

impl PrimaryTag for ArrayType {
    type Tag = Self;
}

impl Serialize<Self> for ArrayType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<TypeId, _>(ArrayTypeField::ElemType, self.elem_type)?;
        serializer.serialize::<tags::U32, _>(ArrayTypeField::Len, self.len)?;

        serializer.finish()
    }
}

impl Serialize<ArrayType> for &ArrayType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ArrayType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut elem_type = None;
        let mut len = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ArrayTypeField::ElemType) => {
                    elem_type = deserializer.deserialize::<TypeId, _>().map(Some)?;
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
