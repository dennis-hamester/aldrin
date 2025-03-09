use super::Reply;
use crate::Error;
use aldrin_core::message::CallFunctionResult;
use futures_channel::oneshot::Receiver;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

/// Future to await the result of a call.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingReply {
    recv: Receiver<Result<(CallFunctionResult, Instant), Error>>,
    id: u32,
    version: Option<u32>,
}

impl PendingReply {
    pub(crate) fn new(
        recv: Receiver<Result<(CallFunctionResult, Instant), Error>>,
        id: u32,
        version: Option<u32>,
    ) -> Self {
        Self { recv, id, version }
    }

    /// Returns the pending reply's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.version
    }

    /// Cast the reply to a typed [`PendingReply<T, E>`](crate::PendingReply).
    pub fn cast<T, E>(self) -> crate::pending_reply::PendingReply<T, E> {
        crate::PendingReply::new(self)
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
            Poll::Ready(Ok(Ok((CallFunctionResult::Ok(t), timestamp)))) => {
                Poll::Ready(Ok(Reply::new(self.id, timestamp, Ok(t))))
            }

            Poll::Ready(Ok(Ok((CallFunctionResult::Err(e), timestamp)))) => {
                Poll::Ready(Ok(Reply::new(self.id, timestamp, Err(e))))
            }

            Poll::Ready(Ok(Ok((CallFunctionResult::Aborted, _)))) => {
                Poll::Ready(Err(Error::CallAborted))
            }

            Poll::Ready(Ok(Ok((CallFunctionResult::InvalidService, _)))) => {
                Poll::Ready(Err(Error::InvalidService))
            }

            Poll::Ready(Ok(Ok((CallFunctionResult::InvalidFunction, _)))) => {
                Poll::Ready(Err(Error::invalid_function(self.id)))
            }

            Poll::Ready(Ok(Ok((CallFunctionResult::InvalidArgs, _)))) => {
                Poll::Ready(Err(Error::invalid_arguments(self.id, None)))
            }

            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(Err(_)) => Poll::Ready(Err(Error::Shutdown)),
            Poll::Pending => Poll::Pending,
        }
    }
}
