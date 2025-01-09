use crate::core::Deserialize;
use crate::error::Error;
use crate::low_level::PendingReply as LlPendingReply;
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
    type Output = Result<Result<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(Ok(reply)) => match reply.into_args() {
                Ok(t) => match t.deserialize() {
                    Ok(t) => Poll::Ready(Ok(Ok(t))),
                    Err(e) => Poll::Ready(Err(Error::invalid_reply(e))),
                },

                Err(e) => match e.deserialize() {
                    Ok(e) => Poll::Ready(Ok(Err(e))),
                    Err(e) => Poll::Ready(Err(Error::invalid_reply(e))),
                },
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
