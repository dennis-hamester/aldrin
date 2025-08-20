use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::{uuid, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewtypeIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) target_type: LexicalId,
}

impl NewtypeIr {
    pub const NAMESPACE: Uuid = uuid!("5269ad99-452b-48a4-96d0-c4a909257d57");

    pub fn new(schema: impl Into<String>, name: impl Into<String>, target_type: LexicalId) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            target_type,
        }
    }

    pub fn lexical_id(&self) -> LexicalId {
        LexicalId::custom(&self.schema, &self.name)
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target_type(&self) -> LexicalId {
        self.target_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum NewtypeField {
    Schema = 0,
    Name = 1,
    TargetType = 2,
}

impl Tag for NewtypeIr {}

impl PrimaryTag for NewtypeIr {
    type Tag = Self;
}

impl Serialize<Self> for NewtypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<NewtypeIr> for &NewtypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(NewtypeField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(NewtypeField::Name, &self.name)?;
        serializer.serialize::<LexicalId, _>(NewtypeField::TargetType, &self.target_type)?;

        serializer.finish()
    }
}

impl Deserialize<Self> for NewtypeIr {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut schema = None;
        let mut name = None;
        let mut target_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(NewtypeField::Schema) => {
                    schema = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(NewtypeField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(NewtypeField::TargetType) => {
                    target_type = deserializer.deserialize::<LexicalId, _>().map(Some)?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            schema: schema.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            target_type: target_type.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
