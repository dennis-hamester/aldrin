use crate::core::message::CallFunctionResult;
use crate::core::SerializedValue;
use crate::error::Error;
use crate::reply::Reply as HlReply;
use futures_channel::oneshot::Receiver;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future to await the result of a call.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Reply {
    recv: Receiver<Result<CallFunctionResult, Error>>,
    function: u32,
}

impl Reply {
    pub(crate) fn new(recv: Receiver<Result<CallFunctionResult, Error>>, function: u32) -> Self {
        Self { recv, function }
    }

    /// Cast the reply to a typed [`Reply<T, E>`](HlReply).
    pub fn cast<T, E>(self) -> crate::reply::Reply<T, E> {
        HlReply::new(self)
    }

    /// Aborts the call and signals that there is no longer interest in the reply.
    ///
    /// This function is equivalent to dropping the `Reply`.
    pub fn abort(self) {}
}

impl Future for Reply {
    type Output = Result<Result<SerializedValue, SerializedValue>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(Ok(CallFunctionResult::Ok(t)))) => Poll::Ready(Ok(Ok(t))),
            Poll::Ready(Ok(Ok(CallFunctionResult::Err(e)))) => Poll::Ready(Ok(Err(e))),

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
