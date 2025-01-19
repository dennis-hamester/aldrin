use std::time::Instant;

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event<T> {
    id: u32,
    timestamp: Instant,
    args: T,
}

impl<T> Event<T> {
    pub(crate) fn new(id: u32, timestamp: Instant, args: T) -> Self {
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

    /// Returns a reference to the event's arguments.
    pub fn args(&self) -> &T {
        &self.args
    }

    /// Returns a mutable reference to the event's arguments.
    pub fn args_mut(&mut self) -> &mut T {
        &mut self.args
    }

    /// Converts the event to its arguments.
    pub fn into_args(self) -> T {
        self.args
    }

    /// Converts from `&Event<T>` to `Event<&T>`.
    pub fn as_ref(&self) -> Event<&T> {
        Event::new(self.id, self.timestamp, &self.args)
    }

    /// Converts from `&mut Event<T>` to `Event<&mut T>`.
    pub fn as_mut(&mut self) -> Event<&mut T> {
        Event::new(self.id, self.timestamp, &mut self.args)
    }
}
