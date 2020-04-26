mod error;
mod event;
mod handle;

use crate::conn_id::ConnectionId;
use aldrin_proto::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_proto::Message;
use futures_channel::mpsc::{Receiver, Sender};
use futures_util::future::{select, Either};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

pub(crate) use event::ConnectionEvent;

pub use error::{ConnectionError, EstablishError};
pub use handle::ConnectionHandle;

#[derive(Debug)]
pub struct Connection<T>
where
    T: AsyncTransport + Unpin,
{
    t: T,
    send: Sender<ConnectionEvent>,
    recv: Receiver<Message>,
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
        recv: Receiver<Message>,
    ) -> Self {
        Connection {
            t,
            send,
            recv,
            handle: Some(ConnectionHandle::new(id)),
        }
    }

    pub fn handle(&self) -> &ConnectionHandle {
        self.handle.as_ref().unwrap()
    }

    pub async fn run(mut self) -> Result<(), ConnectionError<T::Error>> {
        let id = self.handle.take().unwrap().into_id();

        loop {
            match select(self.recv.next(), self.t.receive()).await {
                Either::Left((Some(Message::Shutdown), _)) => {
                    self.t.send_and_flush(Message::Shutdown).await?;
                    self.t.shutdown().await?;
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

                Either::Left((None, _)) => {
                    self.t.shutdown().await?;
                    self.drain_client_recv().await?;
                    return Err(ConnectionError::UnexpectedBrokerShutdown);
                }

                Either::Right((Ok(Some(Message::Shutdown)), _)) => {
                    if let Err(e) = self.send_broker_shutdown(id).await {
                        self.t.shutdown().await?;
                        return Err(e);
                    }

                    if let Err(e) = self.t.shutdown().await {
                        self.drain_broker_recv().await;
                        return Err(e.into());
                    }

                    self.drain_broker_recv().await;
                    return Ok(());
                }

                Either::Right((Ok(Some(msg)), _)) => {
                    if let Err(e) = self.send_broker_msg(id.clone(), msg).await {
                        self.t.shutdown().await?;
                        return Err(e);
                    }
                }

                Either::Right((Ok(None), _)) => {
                    self.send_broker_shutdown(id).await?;
                    self.drain_broker_recv().await;
                    return Err(ConnectionError::UnexpectedClientShutdown);
                }

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
        while let Some(_) = self.recv.next().await {}
    }

    async fn drain_client_recv(&mut self) -> Result<(), ConnectionError<T::Error>> {
        while let Some(_) = self.t.receive().await? {}
        Ok(())
    }
}
