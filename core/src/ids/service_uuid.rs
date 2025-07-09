#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{self, KeyTag, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid};

/// UUID of a service.
///
/// [`ServiceUuid`s](Self) are chosen by the user when creating a service and must be unique among
/// all services of an object.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct ServiceUuid(pub Uuid);

impl ServiceUuid {
    /// Nil `ServiceUuid` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates a [`ServiceUuid`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::ServiceUuid;
    /// let service_uuid = ServiceUuid::new_v4();
    /// ```
    #[cfg(feature = "new-v4-ids")]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(self) -> bool {
        self.0.is_nil()
    }
}

impl Tag for ServiceUuid {}

impl PrimaryTag for ServiceUuid {
    type Tag = Self;
}

impl Serialize<Self> for ServiceUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<ServiceUuid> for &ServiceUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ServiceUuid, _>(*self)
    }
}

impl Deserialize<Self> for ServiceUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for ServiceUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Serialize<tags::Uuid> for &ServiceUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for ServiceUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl KeyTag for ServiceUuid {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for ServiceUuid {
    type KeyTag = Self;
}

impl SerializeKey<Self> for ServiceUuid {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for ServiceUuid {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for ServiceUuid {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for ServiceUuid {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ServiceUuid {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

impl From<Uuid> for ServiceUuid {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ServiceUuid> for Uuid {
    fn from(uuid: ServiceUuid) -> Self {
        uuid.0
    }
}

impl fmt::Display for ServiceUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ServiceUuid {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
