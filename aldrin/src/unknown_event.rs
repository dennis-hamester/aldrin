use crate::low_level;
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Deserialize, DeserializeError, SerializedValueSlice, Value};
use std::error::Error as StdError;
use std::fmt;
use std::time::Instant;

/// An unknown event emitted by a service.
#[derive(Debug, Clone)]
pub struct UnknownEvent {
    inner: low_level::Event,
}

impl UnknownEvent {
    pub(crate) fn new(inner: low_level::Event) -> Self {
        Self { inner }
    }

    /// Extracts the inner low-level event.
    pub fn into_low_level(self) -> low_level::Event {
        self.inner
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the timestamp when the event was received.
    pub fn timestamp(&self) -> Instant {
        self.inner.timestamp()
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.args()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.inner.deserialize_as()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    /// Deserializes the events's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }
}

impl fmt::Display for UnknownEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown event {} received", self.id())
    }
}

impl StdError for UnknownEvent {}
