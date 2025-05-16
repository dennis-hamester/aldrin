use crate::{low_level, Call, Error, Handle};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use std::error::Error as StdError;
use std::fmt;
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
    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
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
    pub fn deserialize_and_cast<A, T, E>(self) -> Result<Call<A, T, E>, Error>
    where
        A: PrimaryTag + Deserialize<A::Tag>,
    {
        self.deserialize_and_cast_as()
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
