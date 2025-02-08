use std::fmt;
use uuid::Uuid;

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

// #[cfg(feature = "introspection")]
// impl Introspectable for ChannelCookie {
//     fn layout() -> Layout {
//         BuiltInType::Uuid.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::UUID
//     }

//     fn add_references(_references: &mut References) {}
// }

// #[cfg(feature = "introspection")]
// impl KeyTypeOf for ChannelCookie {
//     const KEY_TYPE: KeyType = KeyType::Uuid;
// }

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
