use crate::core::message::CallFunctionResult;
use crate::core::Deserialize;
use crate::error::Error;
use futures_channel::oneshot::Receiver;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future to await the result of a call.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Reply<T, E> {
    recv: Receiver<Result<CallFunctionResult, Error>>,
    function: u32,
    phantom: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Reply<T, E> {
    pub(crate) fn new(recv: Receiver<Result<CallFunctionResult, Error>>, function: u32) -> Self {
        Self {
            recv,
            function,
            phantom: PhantomData,
        }
    }
}

impl<T: Deserialize, E: Deserialize> Future for Reply<T, E> {
    type Output = Result<Result<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(Ok(CallFunctionResult::Ok(t)))) => match t.deserialize() {
                Ok(t) => Poll::Ready(Ok(Ok(t))),
                Err(e) => Poll::Ready(Err(Error::invalid_reply(e))),
            },

            Poll::Ready(Ok(Ok(CallFunctionResult::Err(e)))) => match e.deserialize() {
                Ok(e) => Poll::Ready(Ok(Err(e))),
                Err(e) => Poll::Ready(Err(Error::invalid_reply(e))),
            },

            Poll::Ready(Ok(Ok(CallFunctionResult::Aborted))) => {
                Poll::Ready(Err(Error::CallAborted))
            }

            Poll::Ready(Ok(Ok(CallFunctionResult::InvalidService))) => {
                Poll::Ready(Err(Error::InvalidService))
            }

            Poll::Ready(Ok(Ok(CallFunctionResult::InvalidFunction))) => {
                Poll::Ready(Err(Error::invalid_function(self.function)))
            }

            Poll::Ready(Ok(Ok(CallFunctionResult::InvalidArgs))) => {
                Poll::Ready(Err(Error::invalid_arguments(self.function, None)))
            }

            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(Err(_)) => Poll::Ready(Err(Error::Shutdown)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, E> fmt::Debug for Reply<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Reply")
            .field("recv", &self.recv)
            .field("function", &self.function)
            .finish()
    }
}
