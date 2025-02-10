use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Value, ValueKind,
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

impl PrimaryTag for ObjectUuid {
    type Tag = Uuid;
}

impl Serialize<Uuid> for ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Deserialize<Uuid> for ObjectUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<Uuid> for &ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Uuid, _>(*self)
    }
}

impl Serialize<Value> for ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Deserialize<Value> for ObjectUuid {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::Uuid => deserializer.deserialize_uuid().map(Self),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl Serialize<Value> for &ObjectUuid {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Value, _>(*self)
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for ObjectUuid {
//     fn layout() -> Layout {
//         BuiltInType::Uuid.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::UUID
//     }

//     fn add_references(_references: &mut References) {}
// }

// #[cfg(feature = "introspection")]
// impl KeyTypeOf for ObjectUuid {
//     const KEY_TYPE: KeyType = KeyType::Uuid;
// }

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
