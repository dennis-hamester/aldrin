use crate::core::{AsSerializeArg, Serialize, SerializeArg};
use crate::error::Error;
use crate::handle::Handle;
use crate::low_level;
use std::fmt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use std::time::Instant;

/// Replies to a pending call.
pub struct Promise<T: ?Sized, E: ?Sized> {
    inner: low_level::Promise,
    phantom: PhantomData<fn(T, E)>,
}

impl<T: ?Sized, E: ?Sized> Promise<T, E> {
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
    pub fn cast<T2: ?Sized, E2: ?Sized>(self) -> Promise<T2, E2> {
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

impl<T, E> Promise<T, E>
where
    T: AsSerializeArg + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    pub fn ok(self, value: SerializeArg<T>) -> Result<(), Error> {
        self.inner.ok(&value)
    }
}

impl<T, E> Promise<T, E>
where
    T: Serialize + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    pub fn ok_ref(self, value: &T) -> Result<(), Error> {
        self.inner.ok(value)
    }
}

impl<E: ?Sized> Promise<(), E> {
    /// Signals that the call was successful without returning a value.
    pub fn done(self) -> Result<(), Error> {
        self.inner.done()
    }
}

impl<T, E> Promise<T, E>
where
    T: ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Signals that the call failed.
    pub fn err(self, value: SerializeArg<E>) -> Result<(), Error> {
        self.inner.err(&value)
    }
}

impl<T, E> Promise<T, E>
where
    T: ?Sized,
    E: Serialize + ?Sized,
{
    /// Signals that the call failed.
    pub fn err_ref(self, value: &E) -> Result<(), Error> {
        self.inner.err(value)
    }
}

impl<T, E> Promise<T, E>
where
    T: AsSerializeArg + ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Sets the call's reply.
    pub fn set(self, res: Result<SerializeArg<T>, SerializeArg<E>>) -> Result<(), Error> {
        self.inner.set(res.as_ref())
    }
}

impl<T, E> Promise<T, E>
where
    T: Serialize + ?Sized,
    E: Serialize + ?Sized,
{
    /// Sets the call's reply.
    pub fn set_ref(self, res: Result<&T, &E>) -> Result<(), Error> {
        self.inner.set(res)
    }
}

impl<T: ?Sized, E: ?Sized> fmt::Debug for Promise<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Promise")
            .field("inner", &self.inner)
            .finish()
    }
}
