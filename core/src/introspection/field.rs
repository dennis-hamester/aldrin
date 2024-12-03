use super::LexicalId;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    id: u32,
    name: String,
    is_required: bool,
    field_type: LexicalId,
}

impl Field {
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

impl Serialize for Field {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize_field(FieldField::Id, &self.id)?;
        serializer.serialize_field(FieldField::Name, &self.name)?;
        serializer.serialize_field(FieldField::IsRequired, &self.is_required)?;
        serializer.serialize_field(FieldField::FieldType, &self.field_type)?;

        serializer.finish()
    }
}

impl Deserialize for Field {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut is_required = None;
        let mut field_type = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id()? {
                FieldField::Id => id = deserializer.deserialize().map(Some)?,
                FieldField::Name => name = deserializer.deserialize().map(Some)?,
                FieldField::IsRequired => is_required = deserializer.deserialize().map(Some)?,
                FieldField::FieldType => field_type = deserializer.deserialize().map(Some)?,
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
