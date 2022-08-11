#[cfg(test)]
mod test;

use std::time::Instant;

/// Runtime statistics of a broker.
///
/// Some of the statistics refer to the duration between [`start`](Self::start) and
/// [`end`](Self::end). When [`BrokerHandle::take_statistics`](crate::BrokerHandle::take_statistics)
/// is called, these will be reset to 0.
///
/// Also note, that this struct is `non_exhaustive` to make future extensions possible.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BrokerStatistics {
    /// The [`Instant`] when the broker started taking these statistics.
    pub start: Instant,

    /// The [`Instant`] when the broker stopped taking these statistics.
    pub end: Instant,

    /// Number of messages sent by the broker.
    ///
    /// This number is not perfectly accurate. It does not cover messages sent during the handshake
    /// with a new connection and messages sent under certain shutdown scenarios. Overall, only very
    /// few messages are missed.
    pub messages_sent: usize,

    /// Number of messages received from connections.
    ///
    /// This number is not perfectly accurate. It does not cover messages received during the
    /// handshake with a new connection and messages received under certain shutdown scenarios.
    /// Overall, only very few messages are missed.
    pub messages_received: usize,

    /// The number of current connections.
    pub num_connections: usize,

    /// The number of new connections added.
    pub connections_added: usize,

    /// The number of connections shut down.
    pub connections_shut_down: usize,

    /// The number of current objects.
    pub num_objects: usize,

    /// The number of objects created.
    pub objects_created: usize,

    /// The number of objects destroyed.
    pub objects_destroyed: usize,

    /// The number of current services.
    pub num_services: usize,

    /// The number of services created.
    pub services_created: usize,

    /// The number of services destroyed.
    pub services_destroyed: usize,

    /// The number of currently pending function calls.
    pub num_function_calls: usize,

    /// The number of functions called.
    pub functions_called: usize,

    /// The number of functions replied.
    pub functions_replied: usize,

    /// The number of events received by the Broker.
    pub events_received: usize,

    /// The number of events sent by the Broker.
    ///
    /// This number is different from [`events_received`](Self::events_received), because a single
    /// event may be sent out zero, one or multiple times, depending on the number of subscribers.
    pub events_sent: usize,

    /// The number of current channels.
    ///
    /// A channel is counted here as long as at least one end is claimed and not destroyed.
    pub num_channels: usize,

    /// The number of channels created.
    pub channels_created: usize,

    /// The number of channels destroyed.
    pub channels_destroyed: usize,
}

impl BrokerStatistics {
    /// Creates a new [`BrokerStatistics`].
    ///
    /// The timestamps [`start`](Self::start) and [`end`](Self::end) will be initialized with
    /// [`Instant::now()`] (both have the same value). All other fields are initialized to 0.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            end: now,
            messages_sent: 0,
            messages_received: 0,
            num_connections: 0,
            connections_added: 0,
            connections_shut_down: 0,
            num_objects: 0,
            objects_created: 0,
            objects_destroyed: 0,
            num_services: 0,
            services_created: 0,
            services_destroyed: 0,
            num_function_calls: 0,
            functions_called: 0,
            functions_replied: 0,
            events_received: 0,
            events_sent: 0,
            num_channels: 0,
            channels_created: 0,
            channels_destroyed: 0,
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
        self.connections_added = 0;
        self.connections_shut_down = 0;
        self.objects_created = 0;
        self.objects_destroyed = 0;
        self.services_created = 0;
        self.services_destroyed = 0;
        self.functions_called = 0;
        self.functions_replied = 0;
        self.events_received = 0;
        self.events_sent = 0;
        self.channels_created = 0;
        self.channels_destroyed = 0;

        res
    }
}
