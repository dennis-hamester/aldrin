use crate::core::{AsSerializeArg, Serialize, SerializeArg};
use crate::error::Error;
use crate::handle::Handle;
use crate::promise::Promise;
use std::fmt;
use std::task::{Context, Poll};
use std::time::Instant;

/// Pending call.
pub struct Call<Args, T: ?Sized, E: ?Sized> {
    args: Args,
    promise: Promise<T, E>,
}

impl<Args, T: ?Sized, E: ?Sized> Call<Args, T, E> {
    pub(crate) fn new(args: Args, promise: Promise<T, E>) -> Self {
        Self { args, promise }
    }

    /// Returns a handle to the client that was used to create the call.
    pub fn client(&self) -> &Handle {
        self.promise.client()
    }

    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.promise.id()
    }

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.promise.version()
    }

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.promise.timestamp()
    }

    /// Returns a reference to the call's arguments.
    pub fn args(&self) -> &Args {
        &self.args
    }

    /// Returns a mutable reference to the call's arguments.
    pub fn args_mut(&mut self) -> &mut Args {
        &mut self.args
    }

    /// Returns a reference to the call's promise object.
    pub fn promise(&self) -> &Promise<T, E> {
        &self.promise
    }

    /// Returns a mutable reference to the call's promise object.
    pub fn promise_mut(&mut self) -> &mut Promise<T, E> {
        &mut self.promise
    }

    /// Converts the call into a tuple of its arguments and promise.
    pub fn into_args_and_promise(self) -> (Args, Promise<T, E>) {
        (self.args, self.promise)
    }

    /// Casts the call to a different result type.
    pub fn cast<T2: ?Sized, E2: ?Sized>(self) -> Call<Args, T2, E2> {
        Call::new(self.args, self.promise.cast())
    }

    /// Aborts the call.
    ///
    /// The caller will be notified that the call was aborted.
    pub fn abort(self) -> Result<(), Error> {
        self.promise.abort()
    }

    /// Signals that an invalid function was called.
    pub fn invalid_function(self) -> Result<(), Error> {
        self.promise.invalid_function()
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(self) -> Result<(), Error> {
        self.promise.invalid_args()
    }

    /// Returns whether the call was aborted by the caller.
    pub fn is_aborted(&mut self) -> bool {
        self.promise.is_aborted()
    }

    /// Polls whether the call was aborted by the caller.
    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<()> {
        self.promise.poll_aborted(cx)
    }

    /// Resolves if the call was aborted by the caller.
    pub async fn aborted(&mut self) {
        self.promise.aborted().await
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: AsSerializeArg + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    pub fn ok(self, value: SerializeArg<T>) -> Result<(), Error> {
        self.promise.ok(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: Serialize + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    pub fn ok_ref(self, value: &T) -> Result<(), Error> {
        self.promise.ok_ref(value)
    }
}

impl<Args, E: ?Sized> Call<Args, (), E> {
    /// Signals that the call was successful without returning a value.
    pub fn done(self) -> Result<(), Error> {
        self.promise.done()
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Signals that the call failed.
    pub fn err(self, value: SerializeArg<E>) -> Result<(), Error> {
        self.promise.err(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: ?Sized,
    E: Serialize + ?Sized,
{
    /// Signals that the call failed.
    pub fn err_ref(self, value: &E) -> Result<(), Error> {
        self.promise.err_ref(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: AsSerializeArg + ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Sets the call's reply.
    pub fn set(self, res: Result<SerializeArg<T>, SerializeArg<E>>) -> Result<(), Error> {
        self.promise.set(res)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: Serialize + ?Sized,
    E: Serialize + ?Sized,
{
    /// Sets the call's reply.
    pub fn set_ref(self, res: Result<&T, &E>) -> Result<(), Error> {
        self.promise.set_ref(res)
    }
}

impl<Args, T, E> fmt::Debug for Call<Args, T, E>
where
    Args: fmt::Debug,
    T: ?Sized,
    E: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Call")
            .field("args", &self.args)
            .field("promise", &self.promise)
            .finish()
    }
}
