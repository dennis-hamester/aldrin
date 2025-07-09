use super::{ObjectCookie, ObjectUuid};
#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

/// Id of an object.
///
/// [`ObjectId`s][Self] consist of two parts:
/// - An [`ObjectUuid`], identifying the object on the bus
/// - An [`ObjectCookie`], a random UUID chosen by the broker
///
/// It is important to point out, that when an object is destroyed and later created again with the
/// same [`ObjectUuid`], then the [`ObjectCookie`] and consequently the [`ObjectId`] will be
/// different.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct ObjectId {
    /// UUID of the object.
    pub uuid: ObjectUuid,

    /// Cookie of the object.
    pub cookie: ObjectCookie,
}

impl ObjectId {
    /// Nil `ObjectId` (all zeros).
    pub const NIL: Self = Self::new(ObjectUuid::NIL, ObjectCookie::NIL);

    /// Creates a new [`ObjectId`] from an [`ObjectUuid`] and [`ObjectCookie`].
    pub const fn new(uuid: ObjectUuid, cookie: ObjectCookie) -> Self {
        Self { uuid, cookie }
    }

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(self) -> bool {
        self.uuid.is_nil() && self.cookie.is_nil()
    }
}

impl PrimaryTag for ObjectId {
    type Tag = tags::ObjectId;
}

impl Serialize<tags::ObjectId> for ObjectId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_object_id(self)
    }
}

impl Serialize<tags::ObjectId> for &ObjectId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<tags::ObjectId> for ObjectId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_object_id()
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for ObjectId {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::ObjectId.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::OBJECT_ID
    }

    fn add_references(_references: &mut References) {}
}
