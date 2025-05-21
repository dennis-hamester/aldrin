use super::Promise;
use crate::{Error, Handle, UnknownCall};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{
    Deserialize, DeserializeError, Serialize, SerializedValue, SerializedValueSlice, ServiceId,
    Value,
};
use futures_channel::oneshot::Receiver;
use std::task::{Context, Poll};
use std::time::Instant;

/// Pending call.
#[derive(Debug)]
pub struct Call {
    args: SerializedValue,
    promise: Promise,
}

impl Call {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        client: Handle,
        aborted: Receiver<()>,
        serial: u32,
        id: u32,
        version: Option<u32>,
        timestamp: Instant,
        args: SerializedValue,
        service: ServiceId,
    ) -> Self {
        Self {
            args,
            promise: Promise::new(client, id, version, timestamp, aborted, serial, service),
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

    /// Returns the id of the service that the call was received for.
    pub fn service(&self) -> ServiceId {
        self.promise.service()
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
    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.args.deserialize_as()
    }

    /// Deserializes the call's arguments.
    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    /// Deserializes the call's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    /// Returns a reference to the call's promise object.
    pub fn promise(&self) -> &Promise {
        &self.promise
    }

    /// Returns a mutable reference to the call's promise object.
    pub fn promise_mut(&mut self) -> &mut Promise {
        &mut self.promise
    }

    /// Converts this call into its promise object.
    pub fn into_promise(self) -> Promise {
        self.promise
    }

    /// Converts this call into its serialized arguments and a promise object.
    pub fn into_args_and_promise(self) -> (SerializedValue, Promise) {
        (self.args, self.promise)
    }

    /// Deserializes arguments and casts the call to a high-level [`Call`](crate::Call).
    ///
    /// If deserialization fails, then the call will be replied using [`Promise::invalid_args`] and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast_as<K, L, T, E>(self) -> Result<crate::Call<L, T, E>, Error>
    where
        K: Tag,
        L: Deserialize<K>,
    {
        match self.args.deserialize_as() {
            Ok(args) => Ok(crate::Call::new(args, self.promise.cast())),

            Err(e) => {
                let id = self.promise.id();
                let _ = self.promise.invalid_args();
                Err(Error::invalid_arguments(id, Some(e)))
            }
        }
    }

    /// Deserializes arguments and casts the call to a high-level [`Call`](crate::Call).
    ///
    /// If deserialization fails, then the call will be replied using [`Promise::invalid_args`] and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast<A, T, E>(self) -> Result<crate::Call<A, T, E>, Error>
    where
        A: PrimaryTag + Deserialize<A::Tag>,
    {
        self.deserialize_and_cast_as()
    }

    /// Converts this call into an [`UnknownCall`].
    pub fn into_unknown_call(self) -> UnknownCall {
        UnknownCall::new(self)
    }

    /// Sets the call's reply.
    pub fn set_as<T, U, E, F>(self, res: Result<U, F>) -> Result<(), Error>
    where
        T: Tag,
        U: Serialize<T>,
        E: Tag,
        F: Serialize<E>,
    {
        self.promise.set_as(res)
    }

    /// Sets the call's reply.
    pub fn set<T, E>(self, res: Result<T, E>) -> Result<(), Error>
    where
        T: PrimaryTag + Serialize<T::Tag>,
        E: PrimaryTag + Serialize<E::Tag>,
    {
        self.promise.set_as(res)
    }

    /// Signals that the call was successful.
    pub fn ok_as<T: Tag, U: Serialize<T>>(self, value: U) -> Result<(), Error> {
        self.promise.ok_as(value)
    }

    /// Signals that the call was successful.
    pub fn ok<T: PrimaryTag + Serialize<T::Tag>>(self, value: T) -> Result<(), Error> {
        self.promise.ok(value)
    }

    /// Signals that the call was successful without returning a value.
    pub fn done(self) -> Result<(), Error> {
        self.promise.done()
    }

    /// Signals that the call failed.
    pub fn err_as<E: Tag, F: Serialize<E>>(self, value: F) -> Result<(), Error> {
        self.promise.err_as(value)
    }

    /// Signals that the call failed.
    pub fn err<E: PrimaryTag + Serialize<E::Tag>>(self, value: E) -> Result<(), Error> {
        self.promise.err(value)
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

    pub(crate) fn invalid_function_ref(&mut self) {
        self.promise.invalid_function_ref();
    }
}
