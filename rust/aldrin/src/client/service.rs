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

use super::{Error, Handle};
use uuid::Uuid;

#[derive(Debug)]
pub struct Service {
    object_id: Uuid,
    id: Uuid,
    client: Handle,
    destroyed: bool,
}

impl Service {
    pub(crate) fn new(object_id: Uuid, id: Uuid, client: Handle) -> Self {
        Service {
            object_id,
            id,
            client,
            destroyed: false,
        }
    }

    pub async fn destroy(&mut self) -> Result<(), Error> {
        let res = self.client.destroy_service(self.object_id, self.id).await;
        self.destroyed = true;
        res
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if !self.destroyed {
            self.client.destroy_service_now(self.object_id, self.id);
            self.destroyed = true;
        }
    }
}
