use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldIr {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) is_required: bool,
    pub(crate) field_type: LexicalId,
}

impl FieldIr {
    pub(super) fn new(
        id: u32,
        name: impl Into<String>,
        is_required: bool,
        field_type: LexicalId,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            is_required,
            field_type,
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

    pub fn field_type(&self) -> LexicalId {
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

impl Tag for FieldIr {}

impl PrimaryTag for FieldIr {
    type Tag = Self;
}

impl Serialize<Self> for FieldIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<FieldIr> for &FieldIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(FieldField::Id, self.id)?;
        serializer.serialize::<tags::String, _>(FieldField::Name, &self.name)?;
        serializer.serialize::<tags::Bool, _>(FieldField::IsRequired, self.is_required)?;
        serializer.serialize::<LexicalId, _>(FieldField::FieldType, self.field_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for FieldIr {
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
                    field_type = deserializer.deserialize::<LexicalId, _>().map(Some)?
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
