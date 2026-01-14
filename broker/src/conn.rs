mod error;
mod event;
mod handle;
mod select;

use crate::conn_id::ConnectionId;
use aldrin_core::message::{Message, Shutdown};
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt, Buffered};
use futures_channel::mpsc::{Sender, UnboundedReceiver};
use futures_util::sink::SinkExt;
use select::{Select, Selected};

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
    select: Select,
    transport: Option<Buffered<T>>,
    flush_transport: bool,
    send: Sender<ConnectionEvent>,
    recv: Option<UnboundedReceiver<Message>>,
    handle: Option<ConnectionHandle>,
}

impl<T> Connection<T>
where
    T: AsyncTransport + Unpin,
{
    pub(crate) fn new(
        transport: T,
        id: ConnectionId,
        send: Sender<ConnectionEvent>,
        recv: UnboundedReceiver<Message>,
    ) -> Self {
        Self {
            select: Select::new(),
            transport: Some(transport.buffered()),
            flush_transport: false,
            send,
            recv: Some(recv),
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
            debug_assert!(self.recv.is_some());
            debug_assert!(self.transport.is_some());

            match self
                .select
                .select(
                    self.recv.as_mut(),
                    self.transport.as_mut(),
                    self.flush_transport,
                )
                .await
            {
                Selected::Broker(Some(Message::Shutdown(Shutdown))) => {
                    break self.broker_shutdown().await;
                }

                Selected::Broker(Some(msg)) => {
                    if let Err(e) = self.send_message(msg).await {
                        self.client_error(id).await?;
                        break Err(e);
                    }
                }

                Selected::Broker(None) => break Err(ConnectionError::UnexpectedShutdown),

                Selected::Transport(Ok(Message::Shutdown(Shutdown))) => {
                    break self.client_shutdown(id).await;
                }

                Selected::Transport(Ok(msg)) => self.send_broker_msg(id.clone(), msg).await?,

                Selected::TransportFlushed(Ok(())) => self.flush_transport = false,

                Selected::Transport(Err(e)) | Selected::TransportFlushed(Err(e)) => {
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
        msg: impl Into<Message>,
    ) -> Result<(), ConnectionError<T::Error>> {
        if let Some(ref mut transport) = self.transport {
            transport
                .send(msg.into())
                .await
                .map_err(ConnectionError::Transport)?;

            self.flush_transport = true;
        }

        Ok(())
    }

    async fn client_shutdown(&mut self, id: ConnectionId) -> Result<(), ConnectionError<T::Error>> {
        self.send_broker_shutdown(id).await?;
        self.send_message(Shutdown).await?;
        self.drain_broker_recv().await;

        Ok(())
    }

    async fn client_error(&mut self, id: ConnectionId) -> Result<(), ConnectionError<T::Error>> {
        self.transport = None;
        self.flush_transport = false;

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
        while self.recv.is_some() || self.flush_transport {
            match self
                .select
                .select(
                    self.recv.as_mut(),
                    self.transport.as_mut(),
                    self.flush_transport,
                )
                .await
            {
                Selected::Broker(Some(_)) | Selected::Transport(Ok(_)) => {}
                Selected::Broker(None) => self.recv = None,

                Selected::TransportFlushed(Ok(()))
                | Selected::Transport(Err(_))
                | Selected::TransportFlushed(Err(_)) => {
                    self.transport = None;
                    self.flush_transport = false;
                }
            }
        }
    }

    async fn drain_client_recv(&mut self) -> Result<(), ConnectionError<T::Error>> {
        let mut client_shutdown = false;

        while self.transport.is_some() && (!client_shutdown || self.flush_transport) {
            match self
                .select
                .select(
                    self.recv.as_mut(),
                    self.transport.as_mut(),
                    self.flush_transport,
                )
                .await
            {
                Selected::Transport(Ok(Message::Shutdown(Shutdown))) => client_shutdown = true,
                Selected::Broker(Some(_)) | Selected::Transport(Ok(_)) => {}
                Selected::Broker(None) => self.recv = None,
                Selected::TransportFlushed(Ok(())) => self.flush_transport = false,

                Selected::Transport(Err(e)) | Selected::TransportFlushed(Err(e)) => {
                    return Err(ConnectionError::Transport(e));
                }
            }
        }

        Ok(())
    }
}
