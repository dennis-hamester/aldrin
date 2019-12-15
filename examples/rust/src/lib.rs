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

use aldrin::{broker, client, conn};
use aldrin_util::channel::SendError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Error {
    Send(SendError),
    Broker(broker::Error),
    Client(client::Error),
    ConnectionEstablish(conn::EstablishError),
    ConnectionRun(conn::RunError),
    ClientConnect(client::ConnectError),
    ClientRun(client::RunError),
    Join(JoinError),
}

impl From<SendError> for Error {
    fn from(e: SendError) -> Self {
        Error::Send(e)
    }
}

impl From<broker::Error> for Error {
    fn from(e: broker::Error) -> Self {
        Error::Broker(e)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Error::Client(e)
    }
}

impl From<conn::EstablishError> for Error {
    fn from(e: conn::EstablishError) -> Self {
        Error::ConnectionEstablish(e)
    }
}

impl From<conn::RunError> for Error {
    fn from(e: conn::RunError) -> Self {
        Error::ConnectionRun(e)
    }
}

impl From<client::ConnectError> for Error {
    fn from(e: client::ConnectError) -> Self {
        Error::ClientConnect(e)
    }
}

impl From<client::RunError> for Error {
    fn from(e: client::RunError) -> Self {
        Error::ClientRun(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Error::Join(e)
    }
}
