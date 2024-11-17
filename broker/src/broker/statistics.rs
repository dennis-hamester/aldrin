#[cfg(test)]
mod test;

use std::time::Instant;

/// Runtime statistics of a broker.
///
/// Some of the statistics refer to the duration between [`start`](Self::start) and
/// [`end`](Self::end). When [`BrokerHandle::take_statistics`](crate::BrokerHandle::take_statistics)
/// is called, these will be reset to 0.
#[derive(Debug, Clone)]
pub struct BrokerStatistics {
    pub(super) start: Instant,
    pub(super) end: Instant,
    pub(super) messages_sent: usize,
    pub(super) messages_received: usize,
    pub(super) num_connections: usize,
    pub(super) num_objects: usize,
    pub(super) num_services: usize,
    pub(super) num_channels: usize,
    pub(super) num_bus_listeners: usize,

    #[cfg(feature = "introspection")]
    pub(super) num_introspections: usize,
}

impl BrokerStatistics {
    /// Creates a new [`BrokerStatistics`].
    ///
    /// The timestamps [`start`](Self::start) and [`end`](Self::end) will be initialized with
    /// [`Instant::now()`] (both have the same value). All other getters will return 0.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            start: now,
            end: now,
            messages_sent: 0,
            messages_received: 0,
            num_connections: 0,
            num_objects: 0,
            num_services: 0,
            num_channels: 0,
            num_bus_listeners: 0,

            #[cfg(feature = "introspection")]
            num_introspections: 0,
        }
    }

    pub(super) fn take(&mut self) -> Self {
        let now = Instant::now();
        let mut res = self.clone();

        // Fixup timestamps.
        res.end = now;
        self.start = now;

        // Reset statistics to 0.
        self.messages_sent = 0;
        self.messages_received = 0;

        res
    }

    /// The [`Instant`] when the broker started taking these statistics.
    pub fn start(&self) -> Instant {
        self.start
    }

    /// The [`Instant`] when the broker stopped taking these statistics.
    pub fn end(&self) -> Instant {
        self.end
    }

    /// Number of messages sent by the broker.
    ///
    /// This number is not perfectly accurate. It does not cover most messages during the connection
    /// setup and shutdown. Overall, only very few messages are missed.
    pub fn messages_sent(&self) -> usize {
        self.messages_sent
    }

    /// Number of messages received from connections.
    ///
    /// This number is not perfectly accurate. It does not cover most messages during the connection
    /// setup and shutdown. Overall, only very few messages are missed.
    pub fn messages_received(&self) -> usize {
        self.messages_received
    }

    /// The number of current connections.
    pub fn num_connections(&self) -> usize {
        self.num_connections
    }

    /// The number of current objects.
    pub fn num_objects(&self) -> usize {
        self.num_objects
    }

    /// The number of current services.
    pub fn num_services(&self) -> usize {
        self.num_services
    }

    /// The number of current channels.
    ///
    /// A channel is counted here as long as at least one end is claimed and not closed.
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    /// The number of bus listeners.
    pub fn num_bus_listeners(&self) -> usize {
        self.num_bus_listeners
    }

    #[cfg(feature = "introspection")]
    /// The number of registered introspections.
    pub fn num_introspections(&self) -> usize {
        self.num_introspections
    }
}
