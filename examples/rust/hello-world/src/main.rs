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
use aldrin_util::channel::{channel, ClientTransport, ConnectionTransport, SendError};
use tokio::task::JoinError;

const FIFO_SIZE: usize = 16;

async fn broker(t: ConnectionTransport) -> Result<(), Error> {
    let broker = broker::Broker::builder().build();
    let mut handle = broker.handle().clone();
    let join_handle = tokio::spawn(broker.run());

    let conn = handle.add_connection(t).establish::<Error>().await?;
    conn.run::<Error>().await?;

    handle.shutdown().await?;
    join_handle.await?;

    Ok(())
}

async fn client(t: ClientTransport) -> Result<(), Error> {
    client::Client::builder(t).connect::<Error>().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (conn_transport, client_transport) = channel(FIFO_SIZE);

    let broker = tokio::spawn(broker(conn_transport));
    let client = tokio::spawn(client(client_transport));

    broker.await??;
    client.await??;

    Ok(())
}

#[derive(Debug)]
enum Error {
    SendError(SendError),
    BrokerError(broker::Error),
    ConnectionEstablishError(conn::EstablishError),
    ConnectionRunError(conn::RunError),
    ClientConnectError(client::ConnectError),
    ClientRunError(client::RunError),
    JoinError(JoinError),
}

impl From<SendError> for Error {
    fn from(e: SendError) -> Self {
        Error::SendError(e)
    }
}

impl From<broker::Error> for Error {
    fn from(e: broker::Error) -> Self {
        Error::BrokerError(e)
    }
}

impl From<conn::EstablishError> for Error {
    fn from(e: conn::EstablishError) -> Self {
        Error::ConnectionEstablishError(e)
    }
}

impl From<conn::RunError> for Error {
    fn from(e: conn::RunError) -> Self {
        Error::ConnectionRunError(e)
    }
}

impl From<client::ConnectError> for Error {
    fn from(e: client::ConnectError) -> Self {
        Error::ClientConnectError(e)
    }
}

impl From<client::RunError> for Error {
    fn from(e: client::RunError) -> Self {
        Error::ClientRunError(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Error::JoinError(e)
    }
}
