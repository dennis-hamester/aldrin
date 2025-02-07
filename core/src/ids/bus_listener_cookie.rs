#[cfg(feature = "introspection")]
use crate::introspection::{
    BuiltInType, Introspectable, KeyType, KeyTypeOf, Layout, LexicalId, References,
};
use crate::tags::{self, KeyTag, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid};

/// Cookie of a bus listener.
///
/// [`BusListenerCookie`s](Self) are chosen by the broker when creating a bus listener.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct BusListenerCookie(pub Uuid);

impl BusListenerCookie {
    /// Nil `BusListenerCookie` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates a [`BusListenerCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::BusListenerCookie;
    /// let bus_listener_cookie = BusListenerCookie::new_v4();
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

impl Tag for BusListenerCookie {}

impl PrimaryTag for BusListenerCookie {
    type Tag = Self;
}

impl Serialize<Self> for BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<BusListenerCookie> for &BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<BusListenerCookie, _>(*self)
    }
}

impl Deserialize<Self> for BusListenerCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Serialize<tags::Uuid> for &BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for BusListenerCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl KeyTag for BusListenerCookie {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for BusListenerCookie {
    type KeyTag = Self;
}

impl SerializeKey<Self> for BusListenerCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for BusListenerCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for BusListenerCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for BusListenerCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for BusListenerCookie {
    fn layout() -> Layout {
        BuiltInType::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl KeyTypeOf for BusListenerCookie {
    const KEY_TYPE: KeyType = KeyType::Uuid;
}

impl From<Uuid> for BusListenerCookie {
    fn from(cookie: Uuid) -> Self {
        Self(cookie)
    }
}

impl From<BusListenerCookie> for Uuid {
    fn from(cookie: BusListenerCookie) -> Self {
        cookie.0
    }
}

impl fmt::Display for BusListenerCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for BusListenerCookie {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
