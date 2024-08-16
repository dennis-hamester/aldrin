#[cfg(test)]
mod test_old1;

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
    SubscribeAll = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    version: u32,
    type_id: Option<TypeId>,
    subscribe_all: Option<bool>,
}

impl ServiceInfo {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            type_id: None,
            subscribe_all: None,
        }
    }

    pub fn version(self) -> u32 {
        self.version
    }

    #[must_use = "this method follows the builder pattern and returns a new `ServiceInfo`"]
    pub fn set_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn type_id(self) -> Option<TypeId> {
        self.type_id
    }

    #[must_use = "this method follows the builder pattern and returns a new `ServiceInfo`"]
    pub fn set_type_id(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn subscribe_all(self) -> Option<bool> {
        self.subscribe_all
    }

    #[must_use = "this method follows the builder pattern and returns a new `ServiceInfo`"]
    pub fn set_subscribe_all(mut self, subscribe_all: bool) -> Self {
        self.subscribe_all = Some(subscribe_all);
        self
    }
}

impl Serialize for ServiceInfo {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(ServiceInfoField::Version, &self.version)?;
        serializer.serialize_field(ServiceInfoField::TypeId, &self.type_id)?;
        serializer.serialize_field(ServiceInfoField::SubscribeAll, &self.subscribe_all)?;

        serializer.finish()
    }
}

impl Deserialize for ServiceInfo {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut version = None;
        let mut type_id = None;
        let mut subscribe_all = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id() {
                Ok(ServiceInfoField::Version) => version = deserializer.deserialize().map(Some)?,
                Ok(ServiceInfoField::TypeId) => type_id = deserializer.deserialize()?,
                Ok(ServiceInfoField::SubscribeAll) => subscribe_all = deserializer.deserialize()?,
                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish_with(|| {
            Ok(Self {
                version: version.ok_or(DeserializeError::InvalidSerialization)?,
                type_id,
                subscribe_all,
            })
        })
    }
}

#[cfg(test)]
mod test {
    use super::ServiceInfo;
    use crate::ids::TypeId;
    use crate::serialized_value::SerializedValue;
    use uuid::uuid;

    fn serde(info: ServiceInfo) -> ServiceInfo {
        SerializedValue::serialize(&info)
            .unwrap()
            .deserialize()
            .unwrap()
    }

    #[test]
    fn serialize() {
        let info = ServiceInfo::new(1);
        assert_eq!(info, serde(info));

        let info =
            ServiceInfo::new(1).set_type_id(TypeId(uuid!("88e82fb9-03b2-4f51-94d8-4702cfacc90c")));
        assert_eq!(info, serde(info));

        let info = ServiceInfo::new(1).set_subscribe_all(true);
        assert_eq!(info, serde(info));

        let info = ServiceInfo::new(1)
            .set_type_id(TypeId(uuid!("88e82fb9-03b2-4f51-94d8-4702cfacc90c")))
            .set_subscribe_all(true);
        assert_eq!(info, serde(info));
    }
}
