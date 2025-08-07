use crate::handle::request::HandleRequest;
use aldrin_core::message::Message;
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt};
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{future, mem};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Select {
    Transport,
    Handle,
    TransportFlushed,
}

impl Select {
    pub fn new() -> Self {
        Self::Transport
    }

    pub async fn select<T>(
        &mut self,
        transport: &mut T,
        handle: &mut UnboundedReceiver<HandleRequest>,
        flush_transport: bool,
    ) -> Selected<T::Error>
    where
        T: AsyncTransport + Unpin,
    {
        future::poll_fn(|cx| self.poll_select(transport, handle, flush_transport, cx)).await
    }

    fn poll_select<T>(
        &mut self,
        transport: &mut T,
        handle: &mut UnboundedReceiver<HandleRequest>,
        flush_transport: bool,
        cx: &mut Context,
    ) -> Poll<Selected<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        for _ in 0..3 {
            match self.next() {
                Self::Transport => {
                    if let Poll::Ready(res) = transport.receive_poll_unpin(cx) {
                        return Poll::Ready(Selected::Transport(res));
                    }
                }

                Self::Handle => {
                    if let Poll::Ready(res) = Pin::new(&mut *handle).poll_next(cx) {
                        // Unwrap is fine because the client itself holds a sender.
                        return Poll::Ready(Selected::Handle(res.unwrap()));
                    }
                }

                Self::TransportFlushed => {
                    if flush_transport {
                        if let Poll::Ready(res) = transport.send_poll_flush_unpin(cx) {
                            return Poll::Ready(Selected::TransportFlushed(res));
                        }
                    }
                }
            }
        }

        Poll::Pending
    }

    fn next(&mut self) -> Self {
        let next = match self {
            Self::Transport => Self::Handle,
            Self::Handle => Self::TransportFlushed,
            Self::TransportFlushed => Self::Transport,
        };

        mem::replace(self, next)
    }
}

#[derive(Debug)]
pub(crate) enum Selected<E> {
    Transport(Result<Message, E>),
    Handle(HandleRequest),
    TransportFlushed(Result<(), E>),
}
