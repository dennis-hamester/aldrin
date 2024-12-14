use crate::core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice};
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

    /// Converts this event into an [`UnknownEvent`].
    pub fn into_unknown_event(self) -> UnknownEvent {
        UnknownEvent::new(self)
    }
}
