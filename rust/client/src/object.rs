// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
