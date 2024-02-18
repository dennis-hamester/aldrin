use crate::core::message::Message;
use crate::core::transport::{AsyncTransport, AsyncTransportExt};
use crate::function_call_map::FunctionCallMap;
use crate::handle::request::HandleRequest;
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::Stream;
use std::future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Select {
    Transport,
    Handle,
    AbortFunctionCall,
}

impl Select {
    pub fn new() -> Self {
        Self::Transport
    }

    pub async fn select<T>(
        &mut self,
        transport: &mut T,
        handle: &mut UnboundedReceiver<HandleRequest>,
        function_calls: &mut FunctionCallMap,
    ) -> Selected<T>
    where
        T: AsyncTransport + Unpin,
    {
        future::poll_fn(|cx| self.poll_select(transport, handle, function_calls, cx)).await
    }

    fn poll_select<T>(
        &mut self,
        transport: &mut T,
        handle: &mut UnboundedReceiver<HandleRequest>,
        function_calls: &mut FunctionCallMap,
        cx: &mut Context,
    ) -> Poll<Selected<T>>
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

                Self::AbortFunctionCall => {
                    if let Poll::Ready(serial) = function_calls.poll_aborted(cx) {
                        return Poll::Ready(Selected::AbortFunctionCall(serial));
                    }
                }
            }
        }

        Poll::Pending
    }

    fn next(&mut self) -> Self {
        let next = match self {
            Self::Transport => Self::Handle,
            Self::Handle => Self::AbortFunctionCall,
            Self::AbortFunctionCall => Self::Transport,
        };

        mem::replace(self, next)
    }
}

#[derive(Debug)]
pub(crate) enum Selected<T>
where
    T: AsyncTransport,
{
    Transport(Result<Message, T::Error>),
    Handle(HandleRequest),
    AbortFunctionCall(u32),
}
