use crate::error::{DeserializeError, SerializeError};
use crate::ids::TypeId;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceInfoField {
    Version = 0,
    TypeId = 1,
}

#[derive(Debug, Copy, Clone)]
pub struct ServiceInfo {
    pub version: u32,
    pub type_id: Option<TypeId>,
}

impl ServiceInfo {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            type_id: None,
        }
    }

    pub fn with_type_id(version: u32, type_id: TypeId) -> Self {
        Self {
            version,
            type_id: Some(type_id),
        }
    }
}

impl Serialize for ServiceInfo {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ServiceInfoField::Version, &self.version)?;
        serializer.serialize_field(ServiceInfoField::TypeId, &self.type_id)?;

        serializer.finish()
    }
}

impl Deserialize for ServiceInfo {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut version = None;
        let mut type_id = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id() {
                Ok(ServiceInfoField::Version) => version = deserializer.deserialize().map(Some)?,
                Ok(ServiceInfoField::TypeId) => type_id = deserializer.deserialize()?,
                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish_with(|| {
            Ok(Self {
                version: version.ok_or(DeserializeError::InvalidSerialization)?,
                type_id,
            })
        })
    }
}
