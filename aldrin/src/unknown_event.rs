use crate::core::{Deserialize, DeserializeError, SerializedValueSlice};
use crate::low_level::Event;

/// An unknown event emitted by a service.
#[derive(Debug, Clone)]
pub struct UnknownEvent {
    inner: Event,
}

impl UnknownEvent {
    pub(crate) fn new(inner: Event) -> Self {
        Self { inner }
    }

    /// Extracts the inner low-level event.
    pub fn into_low_level(self) -> Event {
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
}
