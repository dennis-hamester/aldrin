use super::Promise;
use crate::call::Call as HlCall;
use crate::core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use crate::error::Error;
use crate::handle::Handle;
use crate::unknown_call::UnknownCall;
use futures_channel::oneshot::Receiver;
use std::time::Instant;

/// Pending call.
#[derive(Debug)]
pub struct Call {
    args: SerializedValue,
    promise: Promise,
}

impl Call {
    pub(crate) fn new(
        client: Handle,
        aborted: Receiver<()>,
        serial: u32,
        id: u32,
        timestamp: Instant,
        args: SerializedValue,
    ) -> Self {
        Self {
            args,
            promise: Promise::new(client, id, timestamp, aborted, serial),
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

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.promise.timestamp()
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

    /// Deserializes the call's arguments.
    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.args.deserialize()
    }

    /// Deserializes the call's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.args.deserialize_as_value()
    }

    /// Converts this call into its promise object.
    pub fn into_promise(self) -> Promise {
        self.promise
    }

    /// Converts this call into its serialized arguments and a promise object.
    pub fn into_args_and_promise(self) -> (SerializedValue, Promise) {
        (self.args, self.promise)
    }

    /// Deserializes arguments and casts the call to a high-level [`Call`](HlCall).
    ///
    /// If deserialization fails, then the call will be replied using [`Promise::invalid_args`] and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast<Args, T, E>(self) -> Result<crate::call::Call<Args, T, E>, Error>
    where
        Args: Deserialize,
        T: ?Sized,
        E: ?Sized,
    {
        match self.args.deserialize() {
            Ok(args) => Ok(HlCall::new(args, self.promise.cast())),

            Err(e) => {
                let id = self.promise.id();
                let _ = self.promise.invalid_args();
                Err(Error::invalid_arguments(id, Some(e)))
            }
        }
    }

    /// Converts this call into an [`UnknownCall`].
    pub fn into_unknown_call(self) -> UnknownCall {
        UnknownCall::new(self)
    }

    pub(crate) fn invalid_function_ref(&mut self) {
        self.promise.invalid_function_ref();
    }
}
