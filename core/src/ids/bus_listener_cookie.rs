use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Value, ValueKind,
};
use std::fmt;
use uuid::Uuid;

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

impl PrimaryTag for BusListenerCookie {
    type Tag = Uuid;
}

impl Serialize<Uuid> for BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Deserialize<Uuid> for BusListenerCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<Uuid> for &BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Uuid, _>(*self)
    }
}

impl Serialize<Value> for BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Deserialize<Value> for BusListenerCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::Uuid => deserializer.deserialize_uuid().map(Self),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl Serialize<Value> for &BusListenerCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Value, _>(*self)
    }
}

// TODO introspection?

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
