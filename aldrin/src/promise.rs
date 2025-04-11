use crate::{low_level, Error, Handle};
use aldrin_core::tags::{self, PrimaryTag};
use aldrin_core::Serialize;
use std::fmt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use std::time::Instant;

/// Replies to a pending call.
pub struct Promise<T, E> {
    inner: low_level::Promise,
    phantom: PhantomData<fn(T, E)>,
}

impl<T, E> Promise<T, E> {
    pub(crate) fn new(inner: low_level::Promise) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns a handle to the client that was used to create the promise.
    pub fn client(&self) -> &Handle {
        self.inner.client()
    }

    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.inner.version()
    }

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.inner.timestamp()
    }

    /// Casts the promise to a different result type.
    pub fn cast<T2, E2>(self) -> Promise<T2, E2> {
        Promise::new(self.inner)
    }

    /// Extracts the inner low-level promise.
    pub fn into_low_level(self) -> low_level::Promise {
        self.inner
    }

    /// Aborts the call.
    ///
    /// The caller will be notified that the call was aborted.
    pub fn abort(self) -> Result<(), Error> {
        self.inner.abort()
    }

    /// Signals that an invalid function was called.
    pub fn invalid_function(self) -> Result<(), Error> {
        self.inner.invalid_function()
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(self) -> Result<(), Error> {
        self.inner.invalid_args()
    }

    /// Returns whether the call was aborted by the caller.
    pub fn is_aborted(&mut self) -> bool {
        self.inner.is_aborted()
    }

    /// Polls whether the call was aborted by the caller.
    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_aborted(cx)
    }

    /// Resolves if the call was aborted by the caller.
    pub async fn aborted(&mut self) {
        self.inner.aborted().await
    }
}

impl<T: PrimaryTag, E> Promise<T, E> {
    /// Signals that the call was successful.
    pub fn ok<U: Serialize<T::Tag>>(self, value: U) -> Result<(), Error> {
        self.inner.ok_as(value)
    }
}

impl<T: PrimaryTag + Serialize<T::Tag>, E> Promise<T, E> {
    /// Signals that the call was successful.
    pub fn ok_val(self, value: T) -> Result<(), Error> {
        self.ok(value)
    }
}

impl<'a, T, E> Promise<T, E>
where
    T: PrimaryTag + 'a,
    &'a T: Serialize<T::Tag>,
{
    /// Signals that the call was successful.
    pub fn ok_ref(self, value: &'a T) -> Result<(), Error> {
        self.ok(value)
    }
}

impl<T: PrimaryTag<Tag = tags::Unit>, E> Promise<T, E> {
    /// Signals that the call was successful without returning a value.
    pub fn done(self) -> Result<(), Error> {
        self.inner.done()
    }
}

impl<T, E: PrimaryTag> Promise<T, E> {
    /// Signals that the call failed.
    pub fn err<F: Serialize<E::Tag>>(self, value: F) -> Result<(), Error> {
        self.inner.err_as(value)
    }
}

impl<T, E: PrimaryTag + Serialize<E::Tag>> Promise<T, E> {
    /// Signals that the call failed.
    pub fn err_val(self, value: E) -> Result<(), Error> {
        self.err(value)
    }
}

impl<'a, T, E> Promise<T, E>
where
    E: PrimaryTag + 'a,
    &'a E: Serialize<E::Tag>,
{
    /// Signals that the call failed.
    pub fn err_ref(self, value: &'a E) -> Result<(), Error> {
        self.err(value)
    }
}

impl<T: PrimaryTag, E: PrimaryTag> Promise<T, E> {
    /// Sets the call's reply.
    pub fn set<U, F>(self, res: Result<U, F>) -> Result<(), Error>
    where
        U: Serialize<T::Tag>,
        F: Serialize<E::Tag>,
    {
        self.inner.set_as(res)
    }
}

impl<T, E> Promise<T, E>
where
    T: PrimaryTag + Serialize<T::Tag>,
    E: PrimaryTag + Serialize<E::Tag>,
{
    /// Sets the call's reply.
    pub fn set_val(self, res: Result<T, E>) -> Result<(), Error> {
        self.set(res)
    }
}

impl<'a, T, E> Promise<T, E>
where
    T: PrimaryTag + 'a,
    &'a T: Serialize<T::Tag>,
    E: PrimaryTag + 'a,
    &'a E: Serialize<E::Tag>,
{
    /// Sets the call's reply.
    pub fn set_ref(self, res: Result<&'a T, &'a E>) -> Result<(), Error> {
        self.set(res)
    }
}

impl<T, E> fmt::Debug for Promise<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Promise")
            .field("inner", &self.inner)
            .finish()
    }
}
