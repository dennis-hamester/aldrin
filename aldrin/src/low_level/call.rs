use super::Promise;
use crate::{Handle, UnknownCall};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use std::time::Instant;

/// Pending call.
#[derive(Debug)]
pub struct Call {
    args: SerializedValue,
    promise: Promise,
}

impl Call {
    pub(crate) fn new(args: SerializedValue, promise: Promise) -> Self {
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

    /// Casts the call to a high-level [`Call`](crate::Call).
    pub fn cast<Args, T, E>(self) -> crate::Call<Args, T, E> {
        crate::Call::new(self.args, self.promise.cast())
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

    /// Converts this call into its promise object.
    pub fn into_promise(self) -> Promise {
        self.promise
    }

    /// Converts this call into its serialized arguments and a promise object.
    pub fn into_args_and_promise(self) -> (SerializedValue, Promise) {
        (self.args, self.promise)
    }

    /// Converts this call into an [`UnknownCall`].
    pub fn into_unknown_call(self) -> UnknownCall {
        UnknownCall::new(self)
    }

    pub(crate) fn invalid_function_ref(&mut self) {
        self.promise.invalid_function_ref();
    }
}
