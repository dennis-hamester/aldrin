use super::{ir, resolve_ir, LexicalId};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Field {
    id: u32,
    name: String,

    #[cfg_attr(feature = "serde", serde(rename = "required"))]
    is_required: bool,

    field_type: TypeId,
}

impl Field {
    pub fn from_ir(ty: ir::FieldIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            id: ty.id,
            name: ty.name,
            is_required: ty.is_required,
            field_type: resolve_ir(ty.field_type, references),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_required(&self) -> bool {
        self.is_required
    }

    pub fn field_type(&self) -> TypeId {
        self.field_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FieldField {
    Id = 0,
    Name = 1,
    IsRequired = 2,
    FieldType = 3,
}

impl Tag for Field {}

impl PrimaryTag for Field {
    type Tag = Self;
}

impl Serialize<Self> for Field {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Field> for &Field {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FieldField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(FieldField::Name, &self.name)?;
        serializer.serialize::<tags::Bool, _>(FieldField::IsRequired, self.is_required)?;
        serializer.serialize::<TypeId, _>(FieldField::FieldType, self.field_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Field {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut is_required = None;
        let mut field_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(FieldField::Id) => id = deserializer.deserialize::<tags::U32, _>().map(Some)?,

                Ok(FieldField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?
                }

                Ok(FieldField::IsRequired) => {
                    is_required = deserializer.deserialize::<tags::Bool, _>().map(Some)?
                }

                Ok(FieldField::FieldType) => {
                    field_type = deserializer.deserialize::<TypeId, _>().map(Some)?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            is_required: is_required.ok_or(DeserializeError::InvalidSerialization)?,
            field_type: field_type.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
