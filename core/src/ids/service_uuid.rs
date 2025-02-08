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

// #[cfg(feature = "introspection")]
// impl Introspectable for ServiceUuid {
//     fn layout() -> Layout {
//         BuiltInType::Uuid.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::UUID
//     }

//     fn add_references(_references: &mut References) {}
// }

// #[cfg(feature = "introspection")]
// impl KeyTypeOf for ServiceUuid {
//     const KEY_TYPE: KeyType = KeyType::Uuid;
// }

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
