use aldrin_core::message::Message;
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt};
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::StreamExt;
use std::task::{Context, Poll};
use std::{future, mem};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Select {
    Broker,
    Transport,
    FlushTransport,
}

impl Select {
    pub fn new() -> Self {
        Self::Broker
    }

    pub async fn select<T>(
        &mut self,
        mut broker: Option<&mut UnboundedReceiver<Message>>,
        mut transport: Option<&mut T>,
        flush_transport: bool,
    ) -> Selected<T::Error>
    where
        T: AsyncTransport + Unpin,
    {
        future::poll_fn(|cx| {
            self.poll_select(
                broker.as_deref_mut(),
                transport.as_deref_mut(),
                flush_transport,
                cx,
            )
        })
        .await
    }

    fn poll_select<T>(
        &mut self,
        mut broker: Option<&mut UnboundedReceiver<Message>>,
        mut transport: Option<&mut T>,
        flush_transport: bool,
        cx: &mut Context,
    ) -> Poll<Selected<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        for _ in 0..3 {
            match self.next() {
                Self::Broker => {
                    if let Some(ref mut broker) = broker {
                        if let Poll::Ready(res) = broker.poll_next_unpin(cx) {
                            return Poll::Ready(Selected::Broker(res));
                        }
                    }
                }

                Self::Transport => {
                    if let Some(ref mut transport) = transport {
                        if let Poll::Ready(res) = transport.receive_poll_unpin(cx) {
                            return Poll::Ready(Selected::Transport(res));
                        }
                    }
                }

                Self::FlushTransport => {
                    if let Some(ref mut transport) = transport {
                        if flush_transport {
                            if let Poll::Ready(res) = transport.send_poll_flush_unpin(cx) {
                                return Poll::Ready(Selected::TransportFlushed(res));
                            }
                        }
                    }
                }
            }
        }

        Poll::Pending
    }

    fn next(&mut self) -> Self {
        let next = match self {
            Self::Broker => Self::Transport,
            Self::Transport => Self::FlushTransport,
            Self::FlushTransport => Self::Broker,
        };

        mem::replace(self, next)
    }
}

pub(crate) enum Selected<E> {
    Broker(Option<Message>),
    Transport(Result<Message, E>),
    TransportFlushed(Result<(), E>),
}
