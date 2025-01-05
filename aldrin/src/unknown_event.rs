use crate::core::{Deserialize, DeserializeError, SerializedValueSlice, Value};
use crate::low_level::Event as LlEvent;
use std::error::Error as StdError;
use std::fmt;

/// An unknown event emitted by a service.
#[derive(Debug, Clone)]
pub struct UnknownEvent {
    inner: LlEvent,
}

impl UnknownEvent {
    pub(crate) fn new(inner: LlEvent) -> Self {
        Self { inner }
    }

    /// Extracts the inner low-level event.
    pub fn into_low_level(self) -> crate::low_level::Event {
        self.inner
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.args()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.inner.deserialize()
    }

    /// Deserializes the events's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.inner.deserialize_as_value()
    }
}

impl fmt::Display for UnknownEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown event {} received", self.id())
    }
}

impl StdError for UnknownEvent {}
