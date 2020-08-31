use super::BrokerShutdown;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::{AsyncTransport, AsyncTransportExt, ConnectReply, Message};
use futures_channel::mpsc::{unbounded, Sender};
use futures_util::sink::SinkExt;

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

    /// Adds a new connection.
    pub async fn add_connection<T>(
        &mut self,
        mut t: T,
    ) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        match t.receive().await? {
            Message::Connect(msg) if msg.version == aldrin_proto::VERSION => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::Ok))
                    .await?;
                Ok(())
            }

            Message::Connect(msg) => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::VersionMismatch(
                    aldrin_proto::VERSION,
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

        Ok(Connection::new(t, id, self.send.clone(), recv))
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

    pub async fn shutdown_connection(
        &mut self,
        conn: &ConnectionHandle,
    ) -> Result<(), BrokerShutdown> {
        self.send
            .send(ConnectionEvent::ShutdownConnection(conn.id().clone()))
            .await
            .map_err(|_| BrokerShutdown)
    }
}
