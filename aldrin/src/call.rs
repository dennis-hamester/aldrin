use crate::{low_level, Error, Handle, Promise, UnknownCall};
use aldrin_core::tags::PrimaryTag;
use aldrin_core::{Deserialize, SerializedValue, SerializedValueSlice};
use std::fmt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use std::time::Instant;

/// Pending call.
pub struct Call<Args, T, E> {
    args: SerializedValue,
    promise: Promise<T, E>,
    phantom: PhantomData<fn() -> Args>,
}

impl<Args, T, E> Call<Args, T, E> {
    pub(crate) fn new(args: SerializedValue, promise: Promise<T, E>) -> Self {
        Self {
            args,
            promise,
            phantom: PhantomData,
        }
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

    /// Casts the call to a different result type.
    pub fn cast<Args2, T2, E2>(self) -> Call<Args2, T2, E2> {
        Call::new(self.args, self.promise.cast())
    }

    /// Converts this call into a low-level [`Call`](low_level::Call).
    pub fn into_low_level(self) -> low_level::Call {
        low_level::Call::new(self.args, self.promise.into_low_level())
    }

    /// Converts this call into an [`UnknownCall`].
    pub fn into_unknown_call(self) -> UnknownCall {
        self.into_low_level().into_unknown_call()
    }

    /// Returns a slice to the call's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        &self.args
    }

    /// Takes out the call's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> SerializedValue {
        self.args.take()
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

impl<Args: PrimaryTag + Deserialize<Args::Tag>, T, E> Call<Args, T, E> {
    /// Deserializes and returns the arguments and the [`Promise`].
    pub fn deserialize(self) -> Result<(Args, Promise<T, E>), Error> {
        self.deserialize_as()
    }
}

impl<Args: PrimaryTag, T, E> Call<Args, T, E> {
    /// Deserializes and returns the arguments and the [`Promise`].
    pub fn deserialize_as<Args2>(self) -> Result<(Args2, Promise<T, E>), Error>
    where
        Args2: Deserialize<Args::Tag>,
    {
        match self.args.deserialize_as() {
            Ok(args) => Ok((args, self.promise)),

            Err(e) => {
                let id = self.id();
                let _ = self.promise.invalid_args();
                Err(Error::invalid_arguments(id, Some(e)))
            }
        }
    }
}

impl<Args, T, E> fmt::Debug for Call<Args, T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Call")
            .field("args", &self.args)
            .field("promise", &self.promise)
            .finish()
    }
}
