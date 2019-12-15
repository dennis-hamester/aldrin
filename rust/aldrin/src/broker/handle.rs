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

use super::Error;
use crate::conn::{self, ConnectionEvent, Transport};
use crate::conn_id::ConnectionIdManager;
use futures_channel::mpsc::Sender;
use futures_util::sink::SinkExt;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct Handle {
    send: Sender<ConnectionEvent>,
    ids: ConnectionIdManager,
}

impl Handle {
    pub(crate) fn new(send: Sender<ConnectionEvent>) -> Handle {
        Handle {
            send,
            ids: ConnectionIdManager::new(),
        }
    }

    pub fn add_connection<T>(&self, t: T) -> conn::Builder<T>
    where
        T: Transport + Unpin,
    {
        conn::Builder::new(t, self.ids.clone(), self.send.clone())
    }

    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.send
            .send(ConnectionEvent::ShutdownBroker)
            .await
            .map_err(Into::into)
    }

    pub async fn shutdown_idle(&mut self) -> Result<(), Error> {
        self.send
            .send(ConnectionEvent::ShutdownIdleBroker)
            .await
            .map_err(Into::into)
    }

    pub fn shutdown_connection<'a>(
        &'a mut self,
        conn: &'_ conn::Handle,
    ) -> impl Future<Output = Result<(), Error>> + 'a {
        let id = conn.id().clone();
        async move {
            self.send
                .send(ConnectionEvent::ShutdownConnection(id))
                .await
                .map_err(Into::into)
        }
    }
}
