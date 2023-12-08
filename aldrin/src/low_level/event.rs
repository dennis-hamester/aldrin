use crate::core::{SerializedValue, ServiceId};

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event {
    /// Id of the service, which emitted the event.
    pub service_id: ServiceId,

    /// Id of the event.
    pub id: u32,

    /// Value of the event.
    pub value: SerializedValue,
}

impl Event {
    /// Creates a new event.
    pub fn new(service_id: ServiceId, id: u32, value: SerializedValue) -> Self {
        Event {
            service_id,
            id,
            value,
        }
    }
}
