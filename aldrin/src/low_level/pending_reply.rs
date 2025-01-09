use super::Reply;
use crate::core::message::CallFunctionResult;
use crate::error::Error;
use crate::pending_reply::PendingReply as HlPendingReply;
use futures_channel::oneshot::Receiver;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future to await the result of a call.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingReply {
    recv: Receiver<Result<CallFunctionResult, Error>>,
    function: u32,
}

impl PendingReply {
    pub(crate) fn new(recv: Receiver<Result<CallFunctionResult, Error>>, function: u32) -> Self {
        Self { recv, function }
    }

    /// Cast the reply to a typed [`PendingReply<T, E>`](HlPendingReply).
    pub fn cast<T, E>(self) -> crate::pending_reply::PendingReply<T, E> {
        HlPendingReply::new(self)
    }

    /// Aborts the call and signals that there is no longer interest in the reply.
    ///
    /// This function is equivalent to dropping the `PendingReply`.
    pub fn abort(self) {}
}

impl Future for PendingReply {
    type Output = Result<Reply, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(Ok(CallFunctionResult::Ok(t)))) => {
                Poll::Ready(Ok(Reply::new(self.function, Ok(t))))
            }

            Poll::Ready(Ok(Ok(CallFunctionResult::Err(e)))) => {
                Poll::Ready(Ok(Reply::new(self.function, Err(e))))
            }

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
