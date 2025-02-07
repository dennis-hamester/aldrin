#[cfg(feature = "introspection")]
use crate::introspection::{
    BuiltInType, Introspectable, KeyType, KeyTypeOf, Layout, LexicalId, References,
};
use crate::tags::{self, KeyTag, PrimaryKeyTag, PrimaryTag, Receiver, Sender, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer,
};
use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid};

/// Cookie of a channel.
///
/// [`ChannelCookie`s](Self) are chosen by the broker when creating a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct ChannelCookie(pub Uuid);

impl ChannelCookie {
    /// Nil `ChannelCookie` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates a [`ChannelCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::ChannelCookie;
    /// let channel_cookie = ChannelCookie::new_v4();
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

impl Tag for ChannelCookie {}

impl PrimaryTag for ChannelCookie {
    type Tag = Self;
}

impl Serialize<Self> for ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<ChannelCookie> for &ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ChannelCookie, _>(*self)
    }
}

impl Deserialize<Self> for ChannelCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Serialize<tags::Uuid> for &ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid, _>(*self)
    }
}

impl Deserialize<tags::Uuid> for ChannelCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl<T: Tag> Serialize<Sender<T>> for ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(self)
    }
}

impl<T: Tag> Serialize<Sender<T>> for &ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Sender<T>, _>(*self)
    }
}

impl<T: Tag> Deserialize<Sender<T>> for ChannelCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender()
    }
}

impl<T: Tag> Serialize<Receiver<T>> for ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(self)
    }
}

impl<T: Tag> Serialize<Receiver<T>> for &ChannelCookie {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Receiver<T>, _>(*self)
    }
}

impl<T: Tag> Deserialize<Receiver<T>> for ChannelCookie {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver()
    }
}

impl KeyTag for ChannelCookie {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for ChannelCookie {
    type KeyTag = Self;
}

impl SerializeKey<Self> for ChannelCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for ChannelCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for ChannelCookie {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for ChannelCookie {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ChannelCookie {
    fn layout() -> Layout {
        BuiltInType::Uuid.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::UUID
    }

    fn add_references(_references: &mut References) {}
}

#[cfg(feature = "introspection")]
impl KeyTypeOf for ChannelCookie {
    const KEY_TYPE: KeyType = KeyType::Uuid;
}

impl From<Uuid> for ChannelCookie {
    fn from(cookie: Uuid) -> Self {
        Self(cookie)
    }
}

impl From<ChannelCookie> for Uuid {
    fn from(cookie: ChannelCookie) -> Self {
        cookie.0
    }
}

impl fmt::Display for ChannelCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ChannelCookie {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
