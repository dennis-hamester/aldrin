use super::BrokerError;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::transport::AsyncTransportExt;
use aldrin_proto::*;
use futures_channel::mpsc::{channel, SendError, Sender};
use futures_util::sink::SinkExt;
use std::future::Future;

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

    pub async fn add_connection<T>(
        &mut self,
        mut t: T,
        fifo_size: usize,
    ) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        match t
            .receive()
            .await?
            .ok_or(EstablishError::UnexpectedClientShutdown)?
        {
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
        let (send, recv) = channel(fifo_size);

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), send))
            .await
            .map_err(|e| {
                if e.is_disconnected() {
                    EstablishError::BrokerShutdown
                } else {
                    EstablishError::InternalError
                }
            })?;

        Ok(Connection::new(t, id, self.send.clone(), recv))
    }

    pub async fn shutdown(&mut self) -> Result<(), BrokerError> {
        self.send
            .send(ConnectionEvent::ShutdownBroker)
            .await
            .map_err(from_send_error)
    }

    pub async fn shutdown_idle(&mut self) -> Result<(), BrokerError> {
        self.send
            .send(ConnectionEvent::ShutdownIdleBroker)
            .await
            .map_err(from_send_error)
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
                .map_err(from_send_error)
        }
    }
}

fn from_send_error(e: SendError) -> BrokerError {
    if e.is_disconnected() {
        BrokerError::BrokerShutdown
    } else {
        BrokerError::InternalError
    }
}
