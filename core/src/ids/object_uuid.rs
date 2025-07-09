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

/// UUID of an object.
///
/// [`ObjectUuid`s](Self) are chosen by the user when creating an object and must be unique among
/// all objects on the bus.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct ObjectUuid(pub Uuid);

impl ObjectUuid {
    /// Nil `ObjectUuid` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates an [`ObjectUuid`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::ObjectUuid;
    /// let object_uuid = ObjectUuid::new_v4();
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

impl Tag for ObjectUuid {}

impl PrimaryTag for ObjectUuid {
    type Tag = Self;
}

impl Serialize<Self> for ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<ObjectUuid> for &ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ObjectUuid, _>(*self)
    }
}

impl Deserialize<Self> for ObjectUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Serialize<tags::Uuid> for &ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for ObjectUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl KeyTag for ObjectUuid {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for ObjectUuid {
    type KeyTag = Self;
}

impl SerializeKey<Self> for ObjectUuid {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for ObjectUuid {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for ObjectUuid {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for ObjectUuid {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ObjectUuid {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

impl From<Uuid> for ObjectUuid {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ObjectUuid> for Uuid {
    fn from(uuid: ObjectUuid) -> Self {
        uuid.0
    }
}

impl fmt::Display for ObjectUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ObjectUuid {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
