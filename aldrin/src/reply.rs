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
    recv: Receiver<CallFunctionResult>,
    function: u32,
    phantom: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Reply<T, E> {
    pub(crate) fn _new(recv: Receiver<CallFunctionResult>, function: u32) -> Self {
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
        let res = match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(_)) => return Poll::Ready(Err(Error::Shutdown)),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(match res {
            CallFunctionResult::Ok(t) => match t.deserialize() {
                Ok(t) => Ok(Ok(t)),
                Err(e) => Err(Error::invalid_reply(e)),
            },

            CallFunctionResult::Err(e) => match e.deserialize() {
                Ok(e) => Ok(Err(e)),
                Err(e) => Err(Error::invalid_reply(e)),
            },

            CallFunctionResult::Aborted => Err(Error::CallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService),
            CallFunctionResult::InvalidFunction => Err(Error::invalid_function(self.function)),
            CallFunctionResult::InvalidArgs => Err(Error::invalid_arguments(self.function, None)),
        })
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
