use super::BrokerError;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::*;
use futures_channel::mpsc::{unbounded, UnboundedSender};
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
    pub async fn add_connection<T>(&self, t: T) -> Result<Connection<T>, EstablishError<T::Error>>
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
        &self,
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
            .unbounded_send(ConnectionEvent::NewConnection(id.clone(), send))
            .map_err(|_| EstablishError::BrokerShutdown)?;

        Ok(Connection::new(t, id, self.send.clone(), recv, fifo_size))
    }

    pub fn shutdown(&self) {
        self.send
            .unbounded_send(ConnectionEvent::ShutdownBroker)
            .ok();
    }

    pub fn shutdown_idle(&self) {
        self.send
            .unbounded_send(ConnectionEvent::ShutdownIdleBroker)
            .ok();
    }

    pub fn shutdown_connection(&self, conn: &ConnectionHandle) -> Result<(), BrokerError> {
        self.send
            .unbounded_send(ConnectionEvent::ShutdownConnection(conn.id().clone()))
            .map_err(|_| BrokerError::BrokerShutdown)
    }
}
