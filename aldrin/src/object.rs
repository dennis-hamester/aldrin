use crate::low_level::{Service, ServiceInfo};
use crate::{Error, Handle, LifetimeId};
use aldrin_core::{ObjectId, ObjectUuid, ServiceUuid};

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
/// use aldrin::core::ObjectUuid;
/// use uuid::uuid;
///
/// const OBJECT2_UUID: ObjectUuid = ObjectUuid(uuid!("6173e119-8066-4776-989b-145a5f16ed4c"));
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut broker = aldrin_test::tokio::TestBroker::new();
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
    /// Creates a new object.
    pub async fn new(client: &Handle, uuid: impl Into<ObjectUuid>) -> Result<Self, Error> {
        client.create_object(uuid).await
    }

    pub(crate) fn new_impl(id: ObjectId, client: Handle) -> Self {
        Self { id, client }
    }

    /// Returns the id of the object.
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns the [`LifetimeId`] associated with the object.
    pub fn lifetime_id(&self) -> LifetimeId {
        self.id.into()
    }

    /// Returns a handle to the client that was used to create the object.
    pub fn client(&self) -> &Handle {
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
    /// use aldrin::Error;
    /// use aldrin::core::{ObjectUuid, ServiceUuid};
    /// use aldrin::low_level::ServiceInfo;
    /// use uuid::uuid;
    ///
    /// const MY_SERVICE_UUID: ServiceUuid = ServiceUuid(uuid!("800b47a1-3882-4601-9155-e18c654476cc"));
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Create a service:
    /// let info = ServiceInfo::new(0);
    /// let service = object.create_service(MY_SERVICE_UUID, info).await?;
    ///
    /// // Trying to create the same service on the same object again will cause an error:
    /// assert_eq!(
    ///     object.create_service(MY_SERVICE_UUID, info).await.unwrap_err(),
    ///     Error::DuplicateService,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_service(
        &self,
        uuid: impl Into<ServiceUuid>,
        info: ServiceInfo,
    ) -> Result<Service, Error> {
        self.client.create_service(self.id, uuid.into(), info).await
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        self.client.destroy_object_now(self.id.cookie);
    }
}
