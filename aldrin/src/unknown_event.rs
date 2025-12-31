use crate::{Event, low_level};
use aldrin_core::tags::Tag;
use aldrin_core::{
    Deserialize, DeserializeError, DeserializePrimary, SerializedValue, SerializedValueSlice,
    ServiceId, Value,
};
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

    /// Returns the id of the service that the event was received for.
    pub fn service(&self) -> ServiceId {
        self.inner.service()
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.args()
    }

    /// Takes out the event's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> SerializedValue {
        self.inner.take_args()
    }

    /// Returns the event's arguments.
    pub fn into_args(self) -> SerializedValue {
        self.inner.into_args()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.inner.deserialize_as()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize<T: DeserializePrimary>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    /// Deserializes the events's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    /// Deserializes the arguments and casts the event to a high-level [`Event`].
    pub fn deserialize_and_cast_as<T: Tag, U: Deserialize<T>>(
        &self,
    ) -> Result<Event<U>, DeserializeError> {
        self.inner.deserialize_and_cast_as()
    }

    /// Deserializes the arguments and casts the event to a high-level [`Event`].
    pub fn deserialize_and_cast<T: DeserializePrimary>(
        &self,
    ) -> Result<Event<T>, DeserializeError> {
        self.inner.deserialize_and_cast()
    }
}

impl fmt::Display for UnknownEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown event {} received", self.id())
    }
}

impl StdError for UnknownEvent {}
