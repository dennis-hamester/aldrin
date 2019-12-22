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

use super::{Client, ConnectError, Transport};
use aldrin_proto::broker::*;
use aldrin_proto::client::*;
use aldrin_proto::{BrokerMessage, ClientMessage, VERSION};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

const DEFAULT_FIFO_SIZE: usize = 16;
const DEFAULT_EVENT_FIFO_SIZE: usize = 16;

#[derive(Debug)]
pub struct Builder<T>
where
    T: Transport + Unpin,
{
    t: T,
    fifo_size: usize,
    event_fifo_size: usize,
}

impl<T> Builder<T>
where
    T: Transport + Unpin,
{
    pub fn new(t: T) -> Self {
        Builder {
            t,
            fifo_size: DEFAULT_FIFO_SIZE,
            event_fifo_size: DEFAULT_EVENT_FIFO_SIZE,
        }
    }

    pub async fn connect<E>(mut self) -> Result<Client<T>, E>
    where
        E: From<ConnectError> + From<T::Error>,
    {
        self.t
            .send(ClientMessage::Connect(Connect { version: VERSION }))
            .await?;

        match self.t.next().await.ok_or(ConnectError::UnexpectedEof)?? {
            BrokerMessage::ConnectReply(ConnectReply::Ok) => {}
            BrokerMessage::ConnectReply(ConnectReply::VersionMismatch(v)) => {
                return Err(ConnectError::VersionMismatch(v).into())
            }
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg).into()),
        }

        Ok(Client::new(self.t, self.fifo_size, self.event_fifo_size))
    }

    pub fn set_fifo_size(mut self, fifo_size: usize) -> Self {
        self.fifo_size = fifo_size;
        self
    }

    pub fn set_event_fifo_size(mut self, event_fifo_size: usize) -> Self {
        self.event_fifo_size = event_fifo_size;
        self
    }
}
