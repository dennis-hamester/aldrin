use super::ServiceInfo;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, SerializedValue,
    Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::uuid;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceInfoFieldOld {
    Version = 0,
    TypeId = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ServiceInfoOld {
    version: u32,
    type_id: Option<TypeId>,
}

impl ServiceInfoOld {
    fn new(version: u32) -> Self {
        Self {
            version,
            type_id: None,
        }
    }

    fn version(self) -> u32 {
        self.version
    }

    fn type_id(self) -> Option<TypeId> {
        self.type_id
    }

    fn set_type_id(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }
}

impl Tag for ServiceInfoOld {}

impl PrimaryTag for ServiceInfoOld {
    type Tag = Self;
}

impl Serialize<Self> for ServiceInfoOld {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize::<tags::U32, _>(ServiceInfoFieldOld::Version, self.version)?;

        serializer
            .serialize::<tags::Option<TypeId>, _>(ServiceInfoFieldOld::TypeId, self.type_id)?;

        serializer.finish()
    }
}

impl Serialize<ServiceInfoOld> for &ServiceInfoOld {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ServiceInfoOld {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut version = None;
        let mut type_id = None;

        while !deserializer.is_empty() {
            let deserializer = deserializer.deserialize()?;

            match deserializer.try_id() {
                Ok(ServiceInfoFieldOld::Version) => {
                    version = deserializer.deserialize::<tags::U32, _>().map(Some)?;
                }

                Ok(ServiceInfoFieldOld::TypeId) => {
                    type_id = deserializer.deserialize::<tags::Option<TypeId>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish_with(|_| {
            Ok(Self {
                version: version.ok_or(DeserializeError::InvalidSerialization)?,
                type_id,
            })
        })
    }
}

#[test]
fn old_to_new() {
    fn serde(old: ServiceInfoOld) -> ServiceInfo {
        SerializedValue::serialize(old)
            .unwrap()
            .deserialize()
            .unwrap()
    }

    let old = ServiceInfoOld::new(1);
    let new = serde(old);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());

    let old =
        ServiceInfoOld::new(1).set_type_id(TypeId(uuid!("88e82fb9-03b2-4f51-94d8-4702cfacc90c")));
    let new = serde(old);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());
}

#[test]
fn new_to_old() {
    fn serde(new: ServiceInfo) -> ServiceInfoOld {
        SerializedValue::serialize(new)
            .unwrap()
            .deserialize()
            .unwrap()
    }

    let new = ServiceInfo::new(1);
    let old = serde(new);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());

    let new =
        ServiceInfo::new(1).set_type_id(TypeId(uuid!("88e82fb9-03b2-4f51-94d8-4702cfacc90c")));
    let old = serde(new);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());

    let new = ServiceInfo::new(1).set_subscribe_all(true);
    let old = serde(new);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());

    let new = ServiceInfo::new(1)
        .set_type_id(TypeId(uuid!("88e82fb9-03b2-4f51-94d8-4702cfacc90c")))
        .set_subscribe_all(true);
    let old = serde(new);
    assert_eq!(old.version(), new.version());
    assert_eq!(old.type_id(), new.type_id());
}
