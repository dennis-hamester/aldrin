use crate::{Call, Error, Handle, low_level};
use aldrin_core::tags::Tag;
use aldrin_core::{
    Deserialize, DeserializeError, DeserializePrimary, Serialize, SerializePrimary,
    SerializedValue, SerializedValueSlice, ServiceId, Value,
};
use std::error::Error as StdError;
use std::fmt;
use std::task::{Context, Poll};
use std::time::Instant;

/// An unknown pending call.
#[derive(Debug)]
pub struct UnknownCall {
    inner: Option<low_level::Call>,
}

impl UnknownCall {
    pub(crate) fn new(inner: low_level::Call) -> Self {
        Self { inner: Some(inner) }
    }

    /// Extracts the inner low-level [`Call`](low_level::Call).
    pub fn into_low_level(mut self) -> low_level::Call {
        self.inner.take().unwrap()
    }

    /// Returns a [`Handle`] to the client that was used to create the call.
    pub fn client(&self) -> &Handle {
        self.inner.as_ref().unwrap().client()
    }

    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.inner.as_ref().unwrap().id()
    }

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.inner.as_ref().unwrap().version()
    }

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.inner.as_ref().unwrap().timestamp()
    }

    /// Returns the id of the service that the call was received for.
    pub fn service(&self) -> ServiceId {
        self.inner.as_ref().unwrap().service()
    }

    /// Returns a slice to the call's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.as_ref().unwrap().args()
    }

    /// Takes out the call's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> SerializedValue {
        self.inner.as_mut().unwrap().take_args()
    }

    /// Deserializes the call's arguments.
    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.inner.as_ref().unwrap().deserialize_as()
    }

    /// Deserializes the call's arguments.
    pub fn deserialize<T: DeserializePrimary>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    /// Deserializes the call's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    /// Returns a reference to the call's promise object.
    pub fn promise(&self) -> &low_level::Promise {
        self.inner.as_ref().unwrap().promise()
    }

    /// Returns a mutable reference to the call's promise object.
    pub fn promise_mut(&mut self) -> &mut low_level::Promise {
        self.inner.as_mut().unwrap().promise_mut()
    }

    /// Converts this call into its promise object.
    pub fn into_promise(mut self) -> low_level::Promise {
        self.inner.take().unwrap().into_promise()
    }

    /// Converts this call into its serialized arguments and a promise object.
    pub fn into_args_and_promise(mut self) -> (SerializedValue, low_level::Promise) {
        self.inner.take().unwrap().into_args_and_promise()
    }

    /// Deserializes arguments and casts the call to a known [`Call`].
    ///
    /// If deserialization fails, then the call will be replied using
    /// [`Promise::invalid_args`](low_level::Promise::invalid_args) and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast_as<K, L, T, E>(mut self) -> Result<Call<L, T, E>, Error>
    where
        K: Tag,
        L: Deserialize<K>,
    {
        self.inner.take().unwrap().deserialize_and_cast_as()
    }

    /// Deserializes arguments and casts the call to a known [`Call`].
    ///
    /// If deserialization fails, then the call will be replied using
    /// [`Promise::invalid_args`](low_level::Promise::invalid_args) and [`Error::InvalidArguments`]
    /// will be returned.
    pub fn deserialize_and_cast<A: DeserializePrimary, T, E>(self) -> Result<Call<A, T, E>, Error> {
        self.deserialize_and_cast_as()
    }

    /// Sets the call's reply.
    pub fn set_as<T, E>(
        mut self,
        res: Result<impl Serialize<T>, impl Serialize<E>>,
    ) -> Result<(), Error>
    where
        T: Tag,
        E: Tag,
    {
        self.inner.take().unwrap().set_as(res)
    }

    /// Sets the call's reply.
    pub fn set(
        mut self,
        res: Result<impl SerializePrimary, impl SerializePrimary>,
    ) -> Result<(), Error> {
        self.inner.take().unwrap().set(res)
    }

    /// Signals that the call was successful.
    pub fn ok_as<T: Tag>(mut self, value: impl Serialize<T>) -> Result<(), Error> {
        self.inner.take().unwrap().ok_as(value)
    }

    /// Signals that the call was successful.
    pub fn ok(mut self, value: impl SerializePrimary) -> Result<(), Error> {
        self.inner.take().unwrap().ok(value)
    }

    /// Signals that the call was successful without returning a value.
    pub fn done(mut self) -> Result<(), Error> {
        self.inner.take().unwrap().done()
    }

    /// Signals that the call failed.
    pub fn err_as<E: Tag>(mut self, value: impl Serialize<E>) -> Result<(), Error> {
        self.inner.take().unwrap().err_as(value)
    }

    /// Signals that the call failed.
    pub fn err(mut self, value: impl SerializePrimary) -> Result<(), Error> {
        self.inner.take().unwrap().err(value)
    }

    /// Aborts the call.
    ///
    /// The caller will be notified that the call was aborted.
    pub fn abort(mut self) -> Result<(), Error> {
        self.inner.take().unwrap().abort()
    }

    /// Signals that an invalid function was called.
    pub fn invalid_function(mut self) -> Result<(), Error> {
        self.inner.take().unwrap().invalid_function()
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(mut self) -> Result<(), Error> {
        self.inner.take().unwrap().invalid_args()
    }

    /// Returns whether the call was aborted by the caller.
    pub fn is_aborted(&mut self) -> bool {
        self.inner.as_mut().unwrap().is_aborted()
    }

    /// Polls whether the call was aborted by the caller.
    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.as_mut().unwrap().poll_aborted(cx)
    }

    /// Resolves if the call was aborted by the caller.
    pub async fn aborted(&mut self) {
        self.inner.as_mut().unwrap().aborted().await
    }
}

impl Drop for UnknownCall {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            inner.invalid_function_ref();
        }
    }
}

impl fmt::Display for UnknownCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown function {} called", self.id())
    }
}

impl StdError for UnknownCall {}
