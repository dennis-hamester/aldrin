mod error;
mod event;
mod handle;

use crate::conn_id::ConnectionId;
use crate::versioned_message::VersionedMessage;
use aldrin_core::message::{Message, Shutdown};
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_core::ProtocolVersion;
use futures_channel::mpsc::{Sender, UnboundedReceiver};
use futures_core::stream::FusedStream;
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

pub(crate) use event::ConnectionEvent;

pub use error::ConnectionError;
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
    transport: T,
    version: ProtocolVersion,
    send: Sender<ConnectionEvent>,
    recv: UnboundedReceiver<VersionedMessage>,
    handle: Option<ConnectionHandle>,
}

impl<T> Connection<T>
where
    T: AsyncTransport + Unpin,
{
    pub(crate) fn new(
        transport: T,
        version: ProtocolVersion,
        id: ConnectionId,
        send: Sender<ConnectionEvent>,
        recv: UnboundedReceiver<VersionedMessage>,
    ) -> Self {
        Self {
            transport,
            version,
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
            match select(self.recv.next(), self.transport.receive()).await {
                Either::Left((
                    Some(VersionedMessage {
                        msg: Message::Shutdown(Shutdown),
                        version: _,
                    }),
                    _,
                )) => {
                    break self.broker_shutdown().await;
                }

                Either::Left((Some(msg), _)) => {
                    if let Err(e) = self.send_message(msg).await {
                        self.send_broker_shutdown(id).await?;
                        self.drain_broker_recv().await;
                        break Err(e);
                    }
                }

                Either::Left((None, _)) => break Err(ConnectionError::UnexpectedShutdown),

                Either::Right((Ok(Message::Shutdown(Shutdown)), _)) => {
                    break self.client_shutdown(id).await
                }

                Either::Right((Ok(msg), _)) => self.send_broker_msg(id.clone(), msg).await?,

                Either::Right((Err(e), _)) => {
                    self.client_error(id).await?;
                    break Err(ConnectionError::Transport(e));
                }
            }
        }
    }

    async fn broker_shutdown(&mut self) -> Result<(), ConnectionError<T::Error>> {
        self.send_message(Shutdown).await?;
        self.drain_client_recv().await?;

        Ok(())
    }

    async fn send_message(
        &mut self,
        msg: impl Into<VersionedMessage>,
    ) -> Result<(), ConnectionError<T::Error>> {
        let msg = msg.into().convert_value(self.version)?;

        self.transport
            .send_and_flush(msg)
            .await
            .map_err(ConnectionError::Transport)
    }

    async fn client_shutdown(&mut self, id: ConnectionId) -> Result<(), ConnectionError<T::Error>> {
        self.send_broker_shutdown(id).await?;
        self.send_message(Shutdown).await?;
        self.drain_broker_recv().await;

        Ok(())
    }

    async fn client_error(&mut self, id: ConnectionId) -> Result<(), ConnectionError<T::Error>> {
        self.send_broker_shutdown(id).await?;
        self.drain_broker_recv().await;

        Ok(())
    }

    async fn send_broker_msg(
        &mut self,
        id: ConnectionId,
        msg: Message,
    ) -> Result<(), ConnectionError<T::Error>> {
        self.send
            .send(ConnectionEvent::Message(id, msg))
            .await
            .map_err(|_| ConnectionError::UnexpectedShutdown)
    }

    async fn send_broker_shutdown(
        &mut self,
        id: ConnectionId,
    ) -> Result<(), ConnectionError<T::Error>> {
        self.send
            .send(ConnectionEvent::ConnectionShutdown(id))
            .await
            .map_err(|_| ConnectionError::UnexpectedShutdown)
    }

    async fn drain_broker_recv(&mut self) {
        while !self.recv.is_terminated() && self.recv.next().await.is_some() {}
    }

    async fn drain_client_recv(&mut self) -> Result<(), ConnectionError<T::Error>> {
        loop {
            let msg = self
                .transport
                .receive()
                .await
                .map_err(ConnectionError::Transport)?;

            if let Message::Shutdown(Shutdown) = msg {
                break Ok(());
            }
        }
    }
}
