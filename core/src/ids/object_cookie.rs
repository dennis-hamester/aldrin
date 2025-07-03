#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, KeyTag, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid};

/// Cookie of an object.
///
/// [`ObjectCookie`s](Self) are chosen by the broker when creating an object. They ensure that
/// objects, created and destroyed over time with the same [`ObjectUuid`](super::ObjectUuid), can
/// still be distinguished.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct ObjectCookie(pub Uuid);

impl ObjectCookie {
    /// Nil `ObjectCookie` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates an [`ObjectCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::ObjectCookie;
    /// let object_cookie = ObjectCookie::new_v4();
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

impl Tag for ObjectCookie {}

impl PrimaryTag for ObjectCookie {
    type Tag = Self;
}

impl Serialize<Self> for ObjectCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<ObjectCookie> for &ObjectCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ObjectCookie, _>(*self)
    }
}

impl Deserialize<Self> for ObjectCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for ObjectCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Serialize<tags::Uuid> for &ObjectCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for ObjectCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl KeyTag for ObjectCookie {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for ObjectCookie {
    type KeyTag = Self;
}

impl SerializeKey<Self> for ObjectCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for ObjectCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for ObjectCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for ObjectCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ObjectCookie {
    fn layout() -> Layout {
        BuiltInType::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

impl From<Uuid> for ObjectCookie {
    fn from(cookie: Uuid) -> Self {
        Self(cookie)
    }
}

impl From<ObjectCookie> for Uuid {
    fn from(cookie: ObjectCookie) -> Self {
        cookie.0
    }
}

impl fmt::Display for ObjectCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ObjectCookie {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
