use crate::{low_level, Error, Reply};
use aldrin_core::tags::PrimaryTag;
use aldrin_core::Deserialize;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future to await the result of a call.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingReply<T, E> {
    inner: low_level::PendingReply,
    phantom: PhantomData<fn() -> (T, E)>,
}

impl<T, E> PendingReply<T, E> {
    pub(crate) fn new(inner: low_level::PendingReply) -> Self {
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

    /// Extracts the inner low-level [`PendingReply`](low_level::PendingReply).
    pub fn into_low_level(self) -> low_level::PendingReply {
        self.inner
    }

    /// Aborts the call and signals that there is no longer interest in the reply.
    ///
    /// This function is equivalent to dropping the `PendingReply`.
    pub fn abort(self) {
        self.inner.abort();
    }
}

impl<T: PrimaryTag, E: PrimaryTag> PendingReply<T, E> {
    /// Polls for the reply.
    pub fn poll_reply_as<U, F>(&mut self, cx: &mut Context) -> Poll<Result<Reply<U, F>, Error>>
    where
        U: Deserialize<T::Tag>,
        F: Deserialize<E::Tag>,
    {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(Ok(reply)) => Poll::Ready(Ok(reply.cast())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, E> Future for PendingReply<T, E>
where
    T: PrimaryTag + Deserialize<T::Tag>,
    E: PrimaryTag + Deserialize<E::Tag>,
{
    type Output = Result<Reply<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.poll_reply_as(cx)
    }
}

impl<T, E> fmt::Debug for PendingReply<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingReply")
            .field("inner", &self.inner)
            .finish()
    }
}
