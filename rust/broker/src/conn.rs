mod error;
mod event;
mod handle;

use crate::broker::BrokerEvent;
use crate::conn_id::ConnectionId;
use aldrin_proto::Transport;
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
    T: Transport + Unpin,
{
    t: T,
    send: Sender<ConnectionEvent>,
    recv: Receiver<BrokerEvent>,
    handle: Option<ConnectionHandle>,
}

impl<T> Connection<T>
where
    T: Transport + Unpin,
{
    pub(crate) fn new(
        t: T,
        id: ConnectionId,
        send: Sender<ConnectionEvent>,
        recv: Receiver<BrokerEvent>,
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

    pub async fn run<E>(mut self) -> Result<(), E>
    where
        E: From<ConnectionError> + From<T::Error>,
    {
        let id = self.handle.take().unwrap().into_id();

        loop {
            match select(self.recv.next(), self.t.next()).await {
                Either::Left((Some(BrokerEvent::Message(msg)), _)) => {
                    if let Err(e) = self.t.send(msg).await {
                        return self.shutdown(id, Err(e)).await;
                    }
                }

                Either::Right((Some(Ok(msg)), _)) => {
                    if let Err(e) = self
                        .send
                        .send(ConnectionEvent::Message(id.clone(), msg))
                        .await
                    {
                        if e.is_disconnected() {
                            return Ok(());
                        } else {
                            return self.shutdown(id, Err(ConnectionError::InternalError)).await;
                        }
                    }
                }

                Either::Left((Some(BrokerEvent::Shutdown), _)) | Either::Left((None, _)) => {
                    return Ok(())
                }

                Either::Right((Some(Err(e)), _)) => return self.shutdown(id, Err(e)).await,
                Either::Right((None, _)) => {
                    return self.shutdown::<ConnectionError, _>(id, Ok(())).await
                }
            }
        }
    }

    async fn shutdown<E1, E2>(mut self, id: ConnectionId, res: Result<(), E1>) -> Result<(), E2>
    where
        E2: From<E1>,
    {
        self.send
            .send(ConnectionEvent::ConnectionShutdown(id))
            .await
            .ok();

        res.map_err(Into::into)
    }
}
