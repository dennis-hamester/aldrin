use super::{Error, Handle, Service, ServiceUuid};
use aldrin_proto::{ConversionError, FromValue, IntoValue, Value};
use std::fmt;
use uuid::Uuid;

/// Owned object on the bus.
///
/// [`Object`s](Object) are created with [`Handle::create_object`] and exist until either manually
/// [destroyed](Object::destroy) or dropped. When an [`Object`] is destroyed, all associated
/// [`Service`s](Service) will be destroyed as well. At runtime, every valid [`Object`] is uniquely
/// identified by an [`ObjectId`] on the bus.
///
/// [`Object`] holds an internal [`Handle`] and will thus prevent the [`Client`](crate::Client) from
/// shutting down automatically. The [`Handle`] is released when the [`Object`] is dropped.
///
/// # Examples
///
/// ```
/// use aldrin_client::ObjectUuid;
///
/// // 6173e119-8066-4776-989b-145a5f16ed4c
/// const OBJECT2_UUID: ObjectUuid = ObjectUuid::from_u128(0x6173e11980664776989b145a5f16ed4c);
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// // Create an object with a random UUID:
/// let object1 = handle.create_object(ObjectUuid::new_v4()).await?;
///
/// // Destroy object1 explicitly:
/// object1.destroy().await?;
///
/// {
///     // Create an object with a fixed UUID:
///     let object2 = handle.create_object(OBJECT2_UUID).await?;
///
///     // object2 is destroyed implicitly when it falls out of scope and is dropped.
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Object {
    id: ObjectId,
    client: Handle,
}

impl Object {
    pub(crate) fn new(id: ObjectId, client: Handle) -> Self {
        Object { id, client }
    }

    /// Returns the id of the object.
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns a handle to the client that was used to create the object.
    pub fn handle(&self) -> &Handle {
        &self.client
    }

    /// Destroys the object.
    ///
    /// If the object has already been destroyed, [`Error::InvalidObject`] is returned.
    pub async fn destroy(&self) -> Result<(), Error> {
        self.client.destroy_object(self.id).await
    }

    /// Creates a service on the object.
    ///
    /// The `uuid` must not yet exists on this [`Object`], or else [`Error::DuplicateService`] will
    /// be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{Error, ObjectUuid, ServiceUuid};
    ///
    /// // 800b47a1-3882-4601-9155-e18c654476cc
    /// const MY_SERVICE_UUID: ServiceUuid = ServiceUuid::from_u128(0x800b47a1388246019155e18c654476cc);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Create a service:
    /// let service = object.create_service(MY_SERVICE_UUID, 0).await?;
    ///
    /// // Trying to create the same service on the same object again will cause an error:
    /// assert_eq!(
    ///     object.create_service(MY_SERVICE_UUID, 0).await.unwrap_err(),
    ///     Error::DuplicateService(object.id(), MY_SERVICE_UUID),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_service(&self, uuid: ServiceUuid, version: u32) -> Result<Service, Error> {
        self.client.create_service(self.id, uuid, version).await
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        self.client.destroy_object_now(self.id.cookie);
    }
}

/// Id of an object.
///
/// An [`ObjectId`] consists of two parts:
/// - An [`ObjectUuid`], identifying the [`Object`] on the bus
/// - An [`ObjectCookie`], a random UUID chosen by the broker
///
/// It is important to point out, that when an object is destroyed and later created again with the
/// same [`ObjectUuid`], then the [`ObjectCookie`] and consequently the [`ObjectId`] will be
/// different. See [`ObjectCookie`] for more information.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId {
    /// UUID of the object.
    pub uuid: ObjectUuid,

    /// Cookie of the object.
    pub cookie: ObjectCookie,
}

impl ObjectId {
    pub(crate) fn new(uuid: ObjectUuid, cookie: ObjectCookie) -> Self {
        ObjectId { uuid, cookie }
    }
}

impl From<aldrin_proto::ObjectId> for ObjectId {
    fn from(id: aldrin_proto::ObjectId) -> Self {
        ObjectId {
            uuid: ObjectUuid(id.uuid),
            cookie: ObjectCookie(id.cookie),
        }
    }
}

impl From<ObjectId> for aldrin_proto::ObjectId {
    fn from(id: ObjectId) -> Self {
        aldrin_proto::ObjectId {
            uuid: id.uuid.0,
            cookie: id.cookie.0,
        }
    }
}

impl FromValue for ObjectId {
    fn from_value(v: Value) -> Result<ObjectId, ConversionError> {
        match v {
            Value::ObjectId(v) => Ok(v.into()),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for ObjectId {
    fn into_value(self) -> Value {
        Value::ObjectId(self.into())
    }
}

/// UUID of an object.
///
/// [`ObjectUuid`s](ObjectUuid) are chosen by the user when [creating](Handle::create_object) an
/// [`Object`] and must be unique among all [`Object`s](Object) on the bus.
///
/// It depends on the use-case whether an [`ObjectUuid`] should be [random](ObjectUuid::new_v4) or
/// [fixed](ObjectUuid::from_u128). As a general rule of thumb, when you're modeling a singleton
/// [`Object`], then chose a fixed [`ObjectUuid`]. This will allow users to find it easily. In all
/// other cases, a random [`ObjectUuid`] is usually the right choice.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectUuid(pub Uuid);

impl ObjectUuid {
    /// Creates an ObjectUuid with a random v4 UUID.
    pub fn new_v4() -> Self {
        ObjectUuid(Uuid::new_v4())
    }

    /// Creates an ObjectUuid from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_client::ObjectUuid;
    /// // b5f0a160-08ea-44ac-b80b-58a9935a96ad
    /// let object_uuid = ObjectUuid::from_u128(0xb5f0a16008ea44acb80b58a9935a96ad);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ObjectUuid(Uuid::from_u128(uuid))
    }
}

impl fmt::Display for ObjectUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Cookie of an object.
///
/// [`ObjectCookie`s](ObjectCookie) are chosen by the broker when [creating](Handle::create_object)
/// an [`Object`]. They help distinguish the [`Object`] across time.
///
/// ```
/// use aldrin_client::ObjectUuid;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// let object_uuid = ObjectUuid::new_v4();
///
/// // Create an object:
/// let object = handle.create_object(object_uuid).await?;
/// let object_id1 = object.id();
/// object.destroy().await?;
///
/// // Create the same object again:
/// let object = handle.create_object(object_uuid).await?;
/// let object_id2 = object.id();
/// object.destroy().await?;
///
/// // The object UUIDs will be equal:
/// assert_eq!(object_id1.uuid, object_id2.uuid);
///
/// // But the cookies will be different:
/// assert_ne!(object_id1.cookie, object_id2.cookie);
///
/// // Consequently, the ids will be different as well:
/// assert_ne!(object_id1, object_id2);
/// # Ok(())
/// # }
/// ```
///
/// In general, [`ObjectCookie`s](ObjectCookie) should be considered an implementation detail of the
/// Aldrin protocol and there is rarely a reason to deal with them manually.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectCookie(pub Uuid);

impl fmt::Display for ObjectCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
