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

use super::{Connection, ConnectionEvent, EstablishError, Transport};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::broker::*;
use aldrin_proto::{BrokerMessage, ClientMessage, VERSION};
use futures_channel::mpsc::{channel, Sender};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

const DEFAULT_FIFO_SIZE: usize = 16;

#[derive(Debug)]
pub struct Builder<T>
where
    T: Transport + Unpin,
{
    t: T,
    ids: ConnectionIdManager,
    send: Sender<ConnectionEvent>,
    fifo_size: usize,
}

impl<T> Builder<T>
where
    T: Transport + Unpin,
{
    pub(crate) fn new(t: T, ids: ConnectionIdManager, send: Sender<ConnectionEvent>) -> Self {
        Builder {
            t,
            ids,
            send,
            fifo_size: DEFAULT_FIFO_SIZE,
        }
    }

    pub async fn establish<E>(mut self) -> Result<Connection<T>, E>
    where
        E: From<EstablishError> + From<T::Error>,
    {
        match self
            .t
            .next()
            .await
            .ok_or(EstablishError::UnexpectedClientShutdown)??
        {
            ClientMessage::Connect(msg) if msg.version == VERSION => {
                self.t
                    .send(BrokerMessage::ConnectReply(ConnectReply::Ok))
                    .await?;
                Ok(())
            }

            ClientMessage::Connect(msg) => {
                self.t
                    .send(BrokerMessage::ConnectReply(ConnectReply::VersionMismatch(
                        VERSION,
                    )))
                    .await?;
                Err(EstablishError::VersionMismatch(msg.version))
            }

            msg => Err(EstablishError::UnexpectedMessageReceived(msg)),
        }?;

        let id = self.ids.acquire();
        let (send, recv) = channel(self.fifo_size);

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), send))
            .await
            .map_err(|e| {
                if e.is_disconnected() {
                    EstablishError::BrokerShutdown
                } else if e.is_full() {
                    EstablishError::BrokerFifoOverflow
                } else {
                    EstablishError::InternalError
                }
            })?;

        Ok(Connection::new(self.t, id, self.send, recv))
    }

    pub fn set_fifo_size(mut self, fifo_size: usize) -> Self {
        self.fifo_size = fifo_size;
        self
    }
}
