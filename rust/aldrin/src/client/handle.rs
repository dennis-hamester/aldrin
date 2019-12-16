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

use super::{Error, Event, Object, ObjectsCreated};
use crate::proto::broker::*;
use futures_channel::mpsc::{channel, Sender};
use futures_channel::oneshot;
use futures_util::sink::SinkExt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Handle {
    send: Sender<Event>,
    event_fifo_size: usize,
}

impl Handle {
    pub(crate) fn new(send: Sender<Event>, event_fifo_size: usize) -> Self {
        Handle {
            send,
            event_fifo_size,
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.send.send(Event::Shutdown).await.map_err(Into::into)
    }

    pub async fn create_object(&mut self, id: Uuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::CreateObject(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::InternalError)?;
        match reply {
            CreateObjectResult::Ok => Ok(Object::new(id, self.clone())),
            CreateObjectResult::DuplicateId => Err(Error::DuplicateObject(id)),
        }
    }

    pub(crate) async fn destroy_object(&mut self, id: Uuid) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send.send(Event::DestroyObject(id, res_send)).await?;
        let reply = res_reply.await.map_err(|_| Error::InternalError)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => Err(Error::InternalError),
        }
    }

    pub async fn objects_created(&mut self, with_current: bool) -> Result<ObjectsCreated, Error> {
        let (ev_send, ev_recv) = channel(self.event_fifo_size);
        self.send
            .send(Event::SubscribeObjectsCreated(ev_send, with_current))
            .await?;
        Ok(ObjectsCreated::new(ev_recv))
    }
}
