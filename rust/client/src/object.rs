use super::{Error, Handle, Service, ServiceUuid};
use std::fmt;
use uuid::Uuid;

/// Owned object.
///
/// Objects are created with [`Handle::create_object`] and exist until either manually
/// [destroyed](Object::destroy) or dropped.
///
/// # Examples
///
/// ```ignore
/// // Create a new object with a random UUID.
/// let mut object = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
///
/// // ...
///
/// // Destroy the object again.
/// object.destroy().await?;
/// ```
#[derive(Debug)]
pub struct Object {
    id: ObjectId,
    client: Option<Handle>,
}

impl Object {
    pub(crate) fn new(id: ObjectId, client: Handle) -> Self {
        Object {
            id,
            client: Some(client),
        }
    }

    /// Returns the id of the object.
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns a handle to the client that was used to create the object.
    ///
    /// `None` is returned after the [`Object`] has been manually [destroyed](Object::destroy).
    pub fn handle(&self) -> Option<&Handle> {
        self.client.as_ref()
    }

    /// Destroys the object.
    ///
    /// If the object has already been destroyed, [`Error::InvalidObject`] is returned.
    pub async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.client.as_mut().ok_or(Error::InvalidObject(self.id))?;
        let res = client.destroy_object(self.id).await;
        if res.is_ok() {
            self.client.take();
        }
        res
    }

    /// Creates a service on the object.
    pub async fn create_service(&mut self, uuid: ServiceUuid) -> Result<Service, Error> {
        let client = self.client.as_mut().ok_or(Error::InvalidObject(self.id))?;
        client.create_service(self.id, uuid).await
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.destroy_object_now(self.id.cookie);
        }
    }
}

/// Id of an object.
///
/// An [`ObjectId`] consists of an arbitrary UUID, and a cookie chosen by the server. The
/// combination of both is unique at all times across the whole Aldrin bus.
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

/// UUID of an object.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectUuid(pub Uuid);

impl fmt::Display for ObjectUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Cookie of an object.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectCookie(pub Uuid);

impl fmt::Display for ObjectCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
