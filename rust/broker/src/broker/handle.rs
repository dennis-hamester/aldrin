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

use super::BrokerError;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::*;
use futures_channel::mpsc::{channel, Sender};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct BrokerHandle {
    send: Sender<ConnectionEvent>,
    ids: ConnectionIdManager,
}

impl BrokerHandle {
    pub(crate) fn new(send: Sender<ConnectionEvent>) -> BrokerHandle {
        BrokerHandle {
            send,
            ids: ConnectionIdManager::new(),
        }
    }

    pub async fn add_connection<T, E>(
        &mut self,
        mut t: T,
        fifo_size: usize,
    ) -> Result<Connection<T>, E>
    where
        T: Transport + Unpin,
        E: From<EstablishError> + From<T::Error>,
    {
        match t
            .next()
            .await
            .ok_or(EstablishError::UnexpectedClientShutdown)??
        {
            Message::Connect(msg) if msg.version == VERSION => {
                t.send(Message::ConnectReply(ConnectReply::Ok)).await?;
                Ok(())
            }

            Message::Connect(msg) => {
                t.send(Message::ConnectReply(ConnectReply::VersionMismatch(
                    VERSION,
                )))
                .await?;
                Err(EstablishError::VersionMismatch(msg.version))
            }

            msg => Err(EstablishError::UnexpectedMessageReceived(msg)),
        }?;

        let id = self.ids.acquire();
        let (send, recv) = channel(fifo_size);

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), send))
            .await
            .map_err(|e| {
                if e.is_disconnected() {
                    EstablishError::BrokerShutdown
                } else {
                    EstablishError::InternalError
                }
            })?;

        Ok(Connection::new(t, id, self.send.clone(), recv))
    }

    pub async fn shutdown(&mut self) -> Result<(), BrokerError> {
        self.send
            .send(ConnectionEvent::ShutdownBroker)
            .await
            .map_err(Into::into)
    }

    pub async fn shutdown_idle(&mut self) -> Result<(), BrokerError> {
        self.send
            .send(ConnectionEvent::ShutdownIdleBroker)
            .await
            .map_err(Into::into)
    }

    pub fn shutdown_connection<'a>(
        &'a mut self,
        conn: &'_ ConnectionHandle,
    ) -> impl Future<Output = Result<(), BrokerError>> + 'a {
        let id = conn.id().clone();
        async move {
            self.send
                .send(ConnectionEvent::ShutdownConnection(id))
                .await
                .map_err(Into::into)
        }
    }
}
