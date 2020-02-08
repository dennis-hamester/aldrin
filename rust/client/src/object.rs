use super::{Error, Handle, Service, ServiceUuid};
use std::fmt;
use uuid::Uuid;

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

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn handle(&self) -> Option<&Handle> {
        self.client.as_ref()
    }

    pub async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.client.as_mut().ok_or(Error::InvalidObject(self.id))?;
        let res = client.destroy_object(self.id).await;
        if res.is_ok() {
            self.client.take();
        }
        res
    }

    pub async fn create_service(
        &mut self,
        uuid: ServiceUuid,
        fifo_size: usize,
    ) -> Result<Service, Error> {
        let client = self.client.as_mut().ok_or(Error::InvalidObject(self.id))?;
        client.create_service(self.id, uuid, fifo_size).await
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.destroy_object_now(self.id.cookie);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ObjectId {
    pub uuid: ObjectUuid,
    pub cookie: ObjectCookie,
}

impl ObjectId {
    pub(crate) fn new(uuid: ObjectUuid, cookie: ObjectCookie) -> Self {
        ObjectId { uuid, cookie }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectUuid(pub Uuid);

impl fmt::Display for ObjectUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ObjectCookie(pub Uuid);

impl fmt::Display for ObjectCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
