use crate::core::{
    Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, ServiceId,
};

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event {
    service: ServiceId,
    id: u32,
    args: SerializedValue,
}

impl Event {
    pub(crate) fn _new(service: ServiceId, id: u32, args: SerializedValue) -> Self {
        Event { service, id, args }
    }

    /// Returns the event's service id.
    pub fn service(&self) -> ServiceId {
        self.service
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
}
