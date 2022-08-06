use crate::{ConversionError, FromValue, IntoValue, Value};
use std::fmt;
use uuid::Uuid;

/// Id of an object.
///
/// [`ObjectId`s][Self] consist of two parts:
/// - An [`ObjectUuid`], identifying the object on the bus
/// - An [`ObjectCookie`], a random UUID chosen by the broker
///
/// It is important to point out, that when an object is destroyed and later created again with the
/// same [`ObjectUuid`], then the [`ObjectCookie`] and consequently the [`ObjectId`] will be
/// different.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ObjectId {
    /// UUID of the object.
    pub uuid: ObjectUuid,

    /// Cookie of the object.
    pub cookie: ObjectCookie,
}

impl ObjectId {
    /// Creates a new [`ObjectId`] from an [`ObjectUuid`] and [`ObjectCookie`].
    pub fn new(uuid: ObjectUuid, cookie: ObjectCookie) -> Self {
        ObjectId { uuid, cookie }
    }
}

impl FromValue for ObjectId {
    fn from_value(v: Value) -> Result<ObjectId, ConversionError> {
        match v {
            Value::ObjectId(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for ObjectId {
    fn into_value(self) -> Value {
        Value::ObjectId(self)
    }
}

/// UUID of an object.
///
/// [`ObjectUuid`s](Self) are chosen by the user when creating an object and must be unique among
/// all objects on the bus.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct ObjectUuid(pub Uuid);

impl ObjectUuid {
    /// Creates an [`ObjectUuid`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ObjectUuid;
    /// let object_uuid = ObjectUuid::new_v4();
    /// ```
    #[cfg(feature = "new-v4-ids")]
    pub fn new_v4() -> Self {
        ObjectUuid(Uuid::new_v4())
    }

    /// Creates an [`ObjectUuid`] from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ObjectUuid;
    /// // b5f0a160-08ea-44ac-b80b-58a9935a96ad
    /// let object_uuid = ObjectUuid::from_u128(0xb5f0a16008ea44acb80b58a9935a96ad);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ObjectUuid(Uuid::from_u128(uuid))
    }
}

impl From<Uuid> for ObjectUuid {
    fn from(uuid: Uuid) -> Self {
        ObjectUuid(uuid)
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

/// Cookie of an object.
///
/// [`ObjectCookie`s](Self) are chosen by the broker when creating an object. They ensure that
/// objects, created and destroyed over time with the same [`ObjectUuid`], can still be
/// distinguished.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct ObjectCookie(pub Uuid);

impl ObjectCookie {
    /// Creates an [`ObjectCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ObjectCookie;
    /// let object_cookie = ObjectCookie::new_v4();
    /// ```
    #[cfg(feature = "new-v4-ids")]
    pub fn new_v4() -> Self {
        ObjectCookie(Uuid::new_v4())
    }

    /// Creates an [`ObjectCookie`] from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ObjectCookie;
    /// // b5f0a160-08ea-44ac-b80b-58a9935a96ad
    /// let object_cookie = ObjectCookie::from_u128(0xb5f0a16008ea44acb80b58a9935a96ad);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ObjectCookie(Uuid::from_u128(uuid))
    }
}

impl From<Uuid> for ObjectCookie {
    fn from(cookie: Uuid) -> Self {
        ObjectCookie(cookie)
    }
}

impl From<ObjectCookie> for Uuid {
    fn from(cookie: ObjectCookie) -> Self {
        cookie.0
    }
}

impl fmt::Display for ObjectCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
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
    /// Creates a new [`ServiceId`] from an [`ObjectId`], a [`ServiceUuid`] and a [`ServiceCookie`].
    pub fn new(object_id: ObjectId, uuid: ServiceUuid, cookie: ServiceCookie) -> Self {
        ServiceId {
            object_id,
            uuid,
            cookie,
        }
    }
}

impl FromValue for ServiceId {
    fn from_value(v: Value) -> Result<ServiceId, ConversionError> {
        match v {
            Value::ServiceId(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for ServiceId {
    fn into_value(self) -> Value {
        Value::ServiceId(self)
    }
}

/// UUID of a service.
///
/// [`ServiceUuid`s](Self) are chosen by the user when creating a service and must be unique among
/// all services of an object.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct ServiceUuid(pub Uuid);

impl ServiceUuid {
    /// Creates a [`ServiceUuid`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ServiceUuid;
    /// let service_uuid = ServiceUuid::new_v4();
    /// ```
    #[cfg(feature = "new-v4-ids")]
    pub fn new_v4() -> Self {
        ServiceUuid(Uuid::new_v4())
    }

    /// Creates a [`ServiceUuid`] from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ServiceUuid;
    /// // b5f0a160-08ea-44ac-b80b-58a9935a96ad
    /// let service_uuid = ServiceUuid::from_u128(0xb5f0a16008ea44acb80b58a9935a96ad);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ServiceUuid(Uuid::from_u128(uuid))
    }
}

impl From<Uuid> for ServiceUuid {
    fn from(uuid: Uuid) -> Self {
        ServiceUuid(uuid)
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

/// Cookie of a service.
///
/// [`ServiceCookie`s](Self) are chosen by the broker when creating a service. They ensure that
/// services, created and destroyed over time with the same [`ServiceUuid`] and on the same object,
/// can still be distinguished.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct ServiceCookie(pub Uuid);

impl ServiceCookie {
    /// Creates a [`ServiceCookie`] with a random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ServiceCookie;
    /// let service_cookie = ServiceCookie::new_v4();
    /// ```
    #[cfg(feature = "new-v4-ids")]
    pub fn new_v4() -> Self {
        ServiceCookie(Uuid::new_v4())
    }

    /// Creates a [`ServiceCookie`] from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ServiceCookie;
    /// // b5f0a160-08ea-44ac-b80b-58a9935a96ad
    /// let service_cookie = ServiceCookie::from_u128(0xb5f0a16008ea44acb80b58a9935a96ad);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ServiceCookie(Uuid::from_u128(uuid))
    }
}

impl From<Uuid> for ServiceCookie {
    fn from(cookie: Uuid) -> Self {
        ServiceCookie(cookie)
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
