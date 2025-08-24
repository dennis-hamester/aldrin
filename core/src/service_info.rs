#[cfg(test)]
mod test_old1;

use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
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

impl Tag for ServiceInfo {}

impl PrimaryTag for ServiceInfo {
    type Tag = Self;
}

impl Serialize<Self> for ServiceInfo {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32>(ServiceInfoField::Version, self.version)?;

        serializer
            .serialize_if_some::<tags::Option<TypeId>>(ServiceInfoField::TypeId, self.type_id)?;

        serializer.serialize_if_some::<tags::Option<tags::Bool>>(
            ServiceInfoField::SubscribeAll,
            self.subscribe_all,
        )?;

        serializer.finish()
    }
}

impl Serialize<ServiceInfo> for &ServiceInfo {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ServiceInfo {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut version = None;
        let mut type_id = None;
        let mut subscribe_all = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(ServiceInfoField::Version) => {
                    version = deserializer.deserialize::<tags::U32, _>().map(Some)?
                }

                Ok(ServiceInfoField::TypeId) => {
                    type_id = deserializer.deserialize::<tags::Option<TypeId>, _>()?
                }

                Ok(ServiceInfoField::SubscribeAll) => {
                    subscribe_all = deserializer.deserialize::<tags::Option<tags::Bool>, _>()?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish_with(|_| {
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
    use crate::{SerializedValue, TypeId};
    use uuid::uuid;

    fn serde(info: ServiceInfo) -> ServiceInfo {
        SerializedValue::serialize(info)
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
