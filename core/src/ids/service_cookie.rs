use std::fmt;
use uuid::Uuid;

/// Cookie of a service.
///
/// [`ServiceCookie`s](Self) are chosen by the broker when creating a service. They ensure that
/// services, created and destroyed over time with the same [`ServiceUuid`] and on the same object,
/// can still be distinguished.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct ServiceCookie(pub Uuid);

impl ServiceCookie {
    /// Nil `ServiceCookie` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Creates a [`ServiceCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_core::ServiceCookie;
    /// let service_cookie = ServiceCookie::new_v4();
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
// impl Introspectable for ServiceCookie {
//     fn layout() -> Layout {
//         BuiltInType::Uuid.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::UUID
//     }

//     fn add_references(_references: &mut References) {}
// }

// #[cfg(feature = "introspection")]
// impl KeyTypeOf for ServiceCookie {
//     const KEY_TYPE: KeyType = KeyType::Uuid;
// }

impl From<Uuid> for ServiceCookie {
    fn from(cookie: Uuid) -> Self {
        Self(cookie)
    }
}

impl From<ServiceCookie> for Uuid {
    fn from(cookie: ServiceCookie) -> Self {
        cookie.0
    }
}

impl fmt::Display for ServiceCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
