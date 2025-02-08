use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid};

/// Introspection type id of a service, struct or enum.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct TypeId(pub Uuid);

impl TypeId {
    /// Nil `TypeId` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(self) -> bool {
        self.0.is_nil()
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for TypeId {
//     fn layout() -> Layout {
//         BuiltInType::Uuid.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::UUID
//     }

//     fn add_references(_references: &mut References) {}
// }

// #[cfg(feature = "introspection")]
// impl KeyTypeOf for TypeId {
//     const KEY_TYPE: KeyType = KeyType::Uuid;
// }

impl From<Uuid> for TypeId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<TypeId> for Uuid {
    fn from(id: TypeId) -> Self {
        id.0
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TypeId {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
