use crate::core::Deserialize;
use crate::error::Error;
use crate::low_level::PendingReply as LlPendingReply;
use crate::reply::Reply;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future to await the result of a call.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingReply<T, E> {
    inner: LlPendingReply,
    phantom: PhantomData<fn() -> (T, E)>,
}

impl<T, E> PendingReply<T, E> {
    pub(crate) fn new(inner: LlPendingReply) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the pending reply's function id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.inner.version()
    }

    /// Casts the reply to resolve to different types.
    pub fn cast<T2, E2>(self) -> PendingReply<T2, E2> {
        PendingReply::new(self.inner)
    }

    /// Extracts the inner low-level [`PendingReply`](LlPendingReply).
    pub fn into_low_level(self) -> crate::low_level::PendingReply {
        self.inner
    }

    /// Aborts the call and signals that there is no longer interest in the reply.
    ///
    /// This function is equivalent to dropping the `PendingReply`.
    pub fn abort(self) {
        self.inner.abort();
    }
}

impl<T: Deserialize, E: Deserialize> Future for PendingReply<T, E> {
    type Output = Result<Reply<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(Ok(reply)) => match reply.deserialize_and_cast() {
                Ok(reply) => Poll::Ready(Ok(reply)),
                Err(e) => Poll::Ready(Err(Error::invalid_reply(e))),
            },

            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, E> fmt::Debug for PendingReply<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingReply")
            .field("inner", &self.inner)
            .finish()
    }
}
