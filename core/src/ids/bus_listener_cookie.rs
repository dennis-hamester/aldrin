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
