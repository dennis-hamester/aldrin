use super::ServiceInfo;
use crate::error::{DeserializeError, SerializeError};
use crate::ids::TypeId;
use crate::serialized_value::SerializedValue;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
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

impl Serialize for ServiceInfoOld {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ServiceInfoFieldOld::Version, &self.version)?;
        serializer.serialize_field(ServiceInfoFieldOld::TypeId, &self.type_id)?;

        serializer.finish()
    }
}

impl Deserialize for ServiceInfoOld {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut version = None;
        let mut type_id = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.try_id() {
                Ok(ServiceInfoFieldOld::Version) => {
                    version = deserializer.deserialize().map(Some)?
                }
                Ok(ServiceInfoFieldOld::TypeId) => type_id = deserializer.deserialize()?,
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

#[test]
fn old_to_new() {
    fn serde(old: ServiceInfoOld) -> ServiceInfo {
        SerializedValue::serialize(&old)
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
        SerializedValue::serialize(&new)
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
