use super::BrokerError;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::transport::AsyncTransportExt;
use aldrin_proto::*;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::sink::SinkExt;
use std::future::Future;
use std::num::NonZeroUsize;

const DEFAULT_FIFO_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct BrokerHandle {
    send: UnboundedSender<ConnectionEvent>,
    ids: ConnectionIdManager,
}

impl BrokerHandle {
    pub(crate) fn new(send: UnboundedSender<ConnectionEvent>) -> BrokerHandle {
        BrokerHandle {
            send,
            ids: ConnectionIdManager::new(),
        }
    }

    /// Adds a connection with the default fifo size.
    ///
    /// The default fifo size is 32. Use [`BrokerHandle::add_connection_with_fifo_size`] to add a
    /// connection with a custom fifo size.
    pub async fn add_connection<T>(
        &mut self,
        t: T,
    ) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        self.add_connection_with_fifo_size(t, NonZeroUsize::new(DEFAULT_FIFO_SIZE))
            .await
    }

    /// Adds a connection with a custom fifo size.
    ///
    /// If `fifo_size` is `None`, then the internal fifo will be unbounded, which should be used
    /// with care. If the fifo overflows, [`Connection::run`] will return immediately with an
    /// error.
    ///
    /// The `fifo_size` parameter affects only outgoing messages. Incoming messages are immediately
    /// passed to the broker.
    pub async fn add_connection_with_fifo_size<T>(
        &mut self,
        mut t: T,
        fifo_size: Option<NonZeroUsize>,
    ) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        match t.receive().await? {
            Message::Connect(msg) if msg.version == VERSION => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::Ok))
                    .await?;
                Ok(())
            }

            Message::Connect(msg) => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::VersionMismatch(
                    VERSION,
                )))
                .await
                .ok();
                Err(EstablishError::VersionMismatch(msg.version))
            }

            msg => Err(EstablishError::UnexpectedMessageReceived(msg)),
        }?;

        let id = self.ids.acquire();
        let (send, recv) = unbounded();

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), send))
            .await
            .map_err(|_| EstablishError::BrokerShutdown)?;

        Ok(Connection::new(t, id, self.send.clone(), recv, fifo_size))
    }

    pub async fn shutdown(&mut self) {
        self.send.send(ConnectionEvent::ShutdownBroker).await.ok();
    }

    pub async fn shutdown_idle(&mut self) {
        self.send
            .send(ConnectionEvent::ShutdownIdleBroker)
            .await
            .ok();
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
                .map_err(|_| BrokerError::BrokerShutdown)
        }
    }
}
