use super::{ObjectId, ServiceCookie, ServiceUuid};
use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Value, ValueKind,
};

/// Id of a service.
///
/// A [`ServiceId`] consists of three parts:
/// - An [`ObjectId`], identifying the associated object on the bus
/// - A [`ServiceUuid`], identifying the service of the object
/// - A [`ServiceCookie`], a random UUID chosen by the broker
///
/// It is important to point out, that when a service is destroyed and later created again with the
/// same [`ServiceUuid`], then the [`ServiceCookie`] and consequently the [`ServiceId`] will be
/// different.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct ServiceId {
    /// Id of the associated object.
    pub object_id: ObjectId,

    /// UUID of the service.
    pub uuid: ServiceUuid,

    /// Cookie of the service.
    pub cookie: ServiceCookie,
}

impl ServiceId {
    /// Nil `ServiceId` (all zeros).
    pub const NIL: Self = Self::new(ObjectId::NIL, ServiceUuid::NIL, ServiceCookie::NIL);

    /// Creates a new [`ServiceId`] from an [`ObjectId`], a [`ServiceUuid`] and a [`ServiceCookie`].
    pub const fn new(object_id: ObjectId, uuid: ServiceUuid, cookie: ServiceCookie) -> Self {
        Self {
            object_id,
            uuid,
            cookie,
        }
    }

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(self) -> bool {
        self.object_id.is_nil() && self.uuid.is_nil() && self.cookie.is_nil()
    }
}

impl PrimaryTag for ServiceId {
    type Tag = Self;
}

impl Serialize<Self> for ServiceId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_service_id(self)
    }
}

impl Deserialize<Self> for ServiceId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_service_id()
    }
}

impl Serialize<ServiceId> for &ServiceId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ServiceId, _>(*self)
    }
}

impl Serialize<Value> for ServiceId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_service_id(self)
    }
}

impl Deserialize<Value> for ServiceId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::ServiceId => deserializer.deserialize_service_id(),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl Serialize<Value> for &ServiceId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Value, _>(*self)
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for ServiceId {
//     fn layout() -> Layout {
//         BuiltInType::ServiceId.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::SERVICE_ID
//     }

//     fn add_references(_references: &mut References) {}
// }
