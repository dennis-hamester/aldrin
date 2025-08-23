use super::LexicalId;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct NewtypeIr {
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) target_type: LexicalId,
}

impl NewtypeIr {
    pub const NAMESPACE: Uuid = uuid!("5269ad99-452b-48a4-96d0-c4a909257d57");

    pub fn builder(
        schema: impl Into<String>,
        name: impl Into<String>,
        target_type: LexicalId,
    ) -> NewtypeIrBuilder {
        NewtypeIrBuilder::new(schema, name, target_type)
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

impl Serialize<NewtypeIr> for &NewtypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::String, _>(NewtypeField::Schema, &self.schema)?;
        serializer.serialize::<tags::String, _>(NewtypeField::Name, &self.name)?;
        serializer.serialize::<LexicalId, _>(NewtypeField::TargetType, &self.target_type)?;

        serializer.finish()
    }
}

#[derive(Debug, Clone)]
pub struct NewtypeIrBuilder {
    schema: String,
    name: String,
    target_type: LexicalId,
}

impl NewtypeIrBuilder {
    pub fn new(schema: impl Into<String>, name: impl Into<String>, target_type: LexicalId) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            target_type,
        }
    }

    pub fn finish(self) -> NewtypeIr {
        NewtypeIr {
            schema: self.schema,
            name: self.name,
            target_type: self.target_type,
        }
    }
}
