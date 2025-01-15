use crate::core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use crate::event::Event as HlEvent;
use crate::unknown_event::UnknownEvent;

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event {
    id: u32,
    args: SerializedValue,
}

impl Event {
    pub(crate) fn new(id: u32, args: SerializedValue) -> Self {
        Self { id, args }
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        &self.args
    }

    /// Returns the event's arguments.
    pub fn into_args(self) -> SerializedValue {
        self.args
    }

    /// Deserializes the event's arguments.
    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.args.deserialize()
    }

    /// Deserializes the event's arguments into a generic [`Value`].
    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.args.deserialize_as_value()
    }

    /// Deserializes the arguments and casts the event to a high-level [`Event`](HlEvent).
    pub fn deserialize_and_cast<T>(&self) -> Result<crate::event::Event<T>, DeserializeError>
    where
        T: Deserialize,
    {
        self.args
            .deserialize()
            .map(|args| HlEvent::new(self.id, args))
    }

    /// Converts this event into an [`UnknownEvent`].
    pub fn into_unknown_event(self) -> UnknownEvent {
        UnknownEvent::new(self)
    }
}
