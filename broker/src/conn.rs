mod error;
mod event;
mod handle;

use crate::conn_id::ConnectionId;
use crate::core::message::{Message, Shutdown};
use crate::core::transport::{AsyncTransport, AsyncTransportExt};
use futures_channel::mpsc::{Sender, UnboundedReceiver};
use futures_core::stream::FusedStream;
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

pub(crate) use event::ConnectionEvent;

pub use error::{ConnectionError, EstablishError};
pub use handle::ConnectionHandle;

/// Connection between a broker and a client.
///
/// `Connection`s can be established with [`BrokerHandle::connect`](crate::BrokerHandle::connect)
/// and must then be [`run`](Connection::run) and polled to completion.
///
/// You can optionally [acquire](Connection::handle) a [`ConnectionHandle`] if you need to be able
/// to shut down specific `Connection`s.
#[derive(Debug)]
#[must_use = "connections do nothing unless you `.await` or poll `Connection::run()`"]
pub struct Connection<T>
where
    T: AsyncTransport + Unpin,
{
    t: T,
    send: Sender<ConnectionEvent>,
    recv: UnboundedReceiver<Message>,
    handle: Option<ConnectionHandle>,
}

impl<T> Connection<T>
where
    T: AsyncTransport + Unpin,
{
    pub(crate) fn new(
        t: T,
        id: ConnectionId,
        send: Sender<ConnectionEvent>,
        recv: UnboundedReceiver<Message>,
    ) -> Self {
        Connection {
            t,
            send,
            recv,
            handle: Some(ConnectionHandle::new(id)),
        }
    }

    /// Returns a reference to the connection handle.
    ///
    /// [`ConnectionHandle`s](ConnectionHandle) can be used to [shut
    /// down](crate::BrokerHandle::shutdown_connection) a specific `Connection`.
    ///
    /// Note also, that this method returns only a reference. However, `ConnectionHandle`s are cheap
    /// to `clone`.
    pub fn handle(&self) -> &ConnectionHandle {
        self.handle.as_ref().unwrap()
    }

    /// Runs the connections.
    ///
    /// After [establishing](crate::BrokerHandle::connect) a new `Connection`, this method must be
    /// called and polled to completion to run the `Connection`.
    pub async fn run(mut self) -> Result<(), ConnectionError<T::Error>> {
        let id = self.handle.take().unwrap().into_id();

        loop {
            match select(self.recv.next(), self.t.receive()).await {
                Either::Left((Some(Message::Shutdown(Shutdown)), _)) => {
                    self.t.send_and_flush(Message::Shutdown(Shutdown)).await?;
                    self.drain_client_recv().await?;
                    return Ok(());
                }

                Either::Left((Some(msg), _)) => {
                    if let Err(e) = self.t.send_and_flush(msg).await {
                        self.send_broker_shutdown(id).await?;
                        self.drain_broker_recv().await;
                        return Err(e.into());
                    }
                }

                Either::Left((None, _)) => return Err(ConnectionError::UnexpectedBrokerShutdown),

                Either::Right((Ok(Message::Shutdown(Shutdown)), _)) => {
                    self.send_broker_shutdown(id).await?;
                    self.t.send_and_flush(Message::Shutdown(Shutdown)).await?;
                    self.drain_broker_recv().await;
                    return Ok(());
                }

                Either::Right((Ok(msg), _)) => self.send_broker_msg(id.clone(), msg).await?,

                Either::Right((Err(e), _)) => {
                    self.send_broker_shutdown(id).await?;
                    self.drain_broker_recv().await;
                    return Err(e.into());
                }
            }
        }
    }

    async fn send_broker_msg(
        &mut self,
        id: ConnectionId,
        msg: Message,
    ) -> Result<(), ConnectionError<T::Error>> {
        self.send
            .send(ConnectionEvent::Message(id, msg))
            .await
            .map_err(|_| ConnectionError::UnexpectedBrokerShutdown)
    }

    async fn send_broker_shutdown(
        &mut self,
        id: ConnectionId,
    ) -> Result<(), ConnectionError<T::Error>> {
        self.send
            .send(ConnectionEvent::ConnectionShutdown(id))
            .await
            .map_err(|_| ConnectionError::UnexpectedBrokerShutdown)
    }

    async fn drain_broker_recv(&mut self) {
        while !self.recv.is_terminated() && self.recv.next().await.is_some() {}
    }

    async fn drain_client_recv(&mut self) -> Result<(), ConnectionError<T::Error>> {
        loop {
            if let Message::Shutdown(Shutdown) = self.t.receive().await? {
                return Ok(());
            }
        }
    }
}
