use super::TypeRef;
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct Field {
    id: u32,
    name: String,
    is_required: bool,
    data: TypeRef,
}

impl Field {
    pub(super) fn new(
        id: u32,
        name: impl Into<String>,
        is_required: bool,
        data: impl Into<TypeRef>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            is_required,
            data: data.into(),
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

    pub fn data(&self) -> &TypeRef {
        &self.data
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FieldField {
    Id = 0,
    Name = 1,
    IsRequired = 2,
    Data = 3,
}

impl Serialize for Field {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize_field(FieldField::Id, &self.id)?;
        serializer.serialize_field(FieldField::Name, &self.name)?;
        serializer.serialize_field(FieldField::IsRequired, &self.is_required)?;
        serializer.serialize_field(FieldField::Data, &self.data)?;

        serializer.finish()
    }
}

impl Deserialize for Field {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_specific_field(FieldField::Id)?;
        let name = deserializer.deserialize_specific_field(FieldField::Name)?;
        let is_required = deserializer.deserialize_specific_field(FieldField::IsRequired)?;
        let data = deserializer.deserialize_specific_field(FieldField::Data)?;

        deserializer.finish(Self {
            id,
            name,
            is_required,
            data,
        })
    }
}
