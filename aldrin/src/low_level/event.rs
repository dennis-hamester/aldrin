use crate::UnknownEvent;
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use std::time::Instant;

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event {
    id: u32,
    timestamp: Instant,
    args: SerializedValue,
}

impl Event {
    pub(crate) fn new(id: u32, timestamp: Instant, args: SerializedValue) -> Self {
        Self {
            id,
            timestamp,
            args,
        }
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the timestamp when the event was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        &self.args
    }

    /// Takes out the event's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> SerializedValue {
        self.args.take()
    }

    /// Returns the event's arguments.
    pub fn into_args(self) -> SerializedValue {
        self.args
    }

    /// Casts the event to a high-level [`Event`](crate::Event).
    pub fn cast<T>(self) -> crate::Event<T> {
        crate::Event::new(self)
    }

    /// Deserializes the event's arguments.
    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.args.deserialize_as()
    }

    /// Deserializes the event's arguments.
    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    /// Deserializes the event's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    /// Converts this event into an [`UnknownEvent`].
    pub fn into_unknown_event(self) -> UnknownEvent {
        UnknownEvent::new(self)
    }
}
