use crate::core::{Deserialize, DeserializeError, SerializedValueSlice, Value};
use crate::error::Error;
use crate::handle::Handle;
use crate::low_level::Call;
use std::error::Error as StdError;
use std::fmt;

/// An unknown pending call.
#[derive(Debug)]
pub struct UnknownCall {
    inner: Option<Call>,
}

impl UnknownCall {
    pub(crate) fn new(inner: Call) -> Self {
        Self { inner: Some(inner) }
    }

    /// Extracts the inner low-level [`Call`].
    pub fn into_low_level(mut self) -> Call {
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

    /// Returns a slice to the call's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.as_ref().unwrap().args()
    }

    /// Deserializes the call's arguments.
    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.inner.as_ref().unwrap().deserialize()
    }

    /// Deserializes the call's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    /// Deserializes arguments and casts the inner promise to a specific set of result types.
    ///
    /// If deserialization fails, then the call will be replied using
    /// [`Promise::invalid_args`](crate::low_level::Promise::invalid_args) and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast<Args, T, E>(
        mut self,
    ) -> Result<(Args, crate::promise::Promise<T, E>), Error>
    where
        Args: Deserialize,
        T: ?Sized,
        E: ?Sized,
    {
        self.inner.take().unwrap().deserialize_and_cast()
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
