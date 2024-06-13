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
    pub(super) connections_added: usize,
    pub(super) connections_shut_down: usize,
    pub(super) num_objects: usize,
    pub(super) objects_created: usize,
    pub(super) objects_destroyed: usize,
    pub(super) num_services: usize,
    pub(super) services_created: usize,
    pub(super) services_destroyed: usize,
    pub(super) num_function_calls: usize,
    pub(super) functions_called: usize,
    pub(super) functions_replied: usize,
    pub(super) events_received: usize,
    pub(super) events_sent: usize,
    pub(super) num_channels: usize,
    pub(super) channels_created: usize,
    pub(super) channels_closed: usize,
    pub(super) items_sent: usize,
    pub(super) items_dropped: usize,
    pub(super) num_bus_listeners: usize,
    pub(super) bus_listeners_created: usize,
    pub(super) bus_listeners_destroyed: usize,
    pub(super) num_bus_listeners_active: usize,
    pub(super) bus_listeners_started: usize,
    pub(super) bus_listeners_stopped: usize,
    pub(super) bus_listener_filters_added: usize,
    pub(super) bus_listener_filters_removed: usize,
    pub(super) bus_listener_filters_cleared: usize,
    pub(super) bus_events_sent: usize,
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
            channels_closed: 0,
            items_sent: 0,
            items_dropped: 0,
            num_bus_listeners: 0,
            bus_listeners_created: 0,
            bus_listeners_destroyed: 0,
            num_bus_listeners_active: 0,
            bus_listeners_started: 0,
            bus_listeners_stopped: 0,
            bus_listener_filters_added: 0,
            bus_listener_filters_removed: 0,
            bus_listener_filters_cleared: 0,
            bus_events_sent: 0,
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
        self.channels_closed = 0;
        self.items_sent = 0;
        self.items_dropped = 0;
        self.bus_listeners_created = 0;
        self.bus_listeners_destroyed = 0;
        self.bus_listeners_started = 0;
        self.bus_listeners_stopped = 0;
        self.bus_listener_filters_added = 0;
        self.bus_listener_filters_removed = 0;
        self.bus_listener_filters_cleared = 0;
        self.bus_events_sent = 0;

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

    /// The number of new connections added.
    pub fn connections_added(&self) -> usize {
        self.connections_added
    }

    /// The number of connections shut down.
    pub fn connections_shut_down(&self) -> usize {
        self.connections_shut_down
    }

    /// The number of current objects.
    pub fn num_objects(&self) -> usize {
        self.num_objects
    }

    /// The number of objects created.
    pub fn objects_created(&self) -> usize {
        self.objects_created
    }

    /// The number of objects destroyed.
    pub fn objects_destroyed(&self) -> usize {
        self.objects_destroyed
    }

    /// The number of current services.
    pub fn num_services(&self) -> usize {
        self.num_services
    }

    /// The number of services created.
    pub fn services_created(&self) -> usize {
        self.services_created
    }

    /// The number of services destroyed.
    pub fn services_destroyed(&self) -> usize {
        self.services_destroyed
    }

    /// The number of currently pending function calls.
    pub fn num_function_calls(&self) -> usize {
        self.num_function_calls
    }

    /// The number of functions called.
    pub fn functions_called(&self) -> usize {
        self.functions_called
    }

    /// The number of functions replied.
    pub fn functions_replied(&self) -> usize {
        self.functions_replied
    }

    /// The number of events received by the broker.
    pub fn events_received(&self) -> usize {
        self.events_received
    }

    /// The number of events sent by the broker.
    ///
    /// This number is different from [`events_received`](Self::events_received), because a single
    /// event may be sent out zero, one or multiple times, depending on the number of subscribers.
    pub fn events_sent(&self) -> usize {
        self.events_sent
    }

    /// The number of current channels.
    ///
    /// A channel is counted here as long as at least one end is claimed and not closed.
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    /// The number of channels created.
    pub fn channels_created(&self) -> usize {
        self.channels_created
    }

    /// The number of channels closed.
    pub fn channels_closed(&self) -> usize {
        self.channels_closed
    }

    /// The number of items sent successfully on a channel.
    pub fn items_sent(&self) -> usize {
        self.items_sent
    }

    /// The number of items dropped a channel.
    pub fn items_dropped(&self) -> usize {
        self.items_dropped
    }

    /// The number of bus listeners.
    pub fn num_bus_listeners(&self) -> usize {
        self.num_bus_listeners
    }

    /// The number of bus listeners created.
    pub fn bus_listeners_created(&self) -> usize {
        self.bus_listeners_created
    }

    /// The number of bus listeners destroyed.
    pub fn bus_listeners_destroyed(&self) -> usize {
        self.bus_listeners_destroyed
    }

    /// The number of bus listeners that are active / have been started.
    pub fn num_bus_listeners_active(&self) -> usize {
        self.num_bus_listeners_active
    }

    /// The number of bus listeners started.
    pub fn bus_listeners_started(&self) -> usize {
        self.bus_listeners_started
    }

    /// The number of bus listeners stopped.
    pub fn bus_listeners_stopped(&self) -> usize {
        self.bus_listeners_stopped
    }

    /// The number of bus listener filters added.
    pub fn bus_listener_filters_added(&self) -> usize {
        self.bus_listener_filters_added
    }

    /// The number of bus listener filters removed.
    pub fn bus_listener_filters_removed(&self) -> usize {
        self.bus_listener_filters_removed
    }

    /// The number of bus listener filters cleared.
    pub fn bus_listener_filters_cleared(&self) -> usize {
        self.bus_listener_filters_cleared
    }

    /// The number of bus events sent.
    ///
    /// When interpreting this statistic, take note that bus events are sent only once per client,
    /// not per bus listener that matches it. Clients then dispatch bus events to individual bus
    /// listeners.
    pub fn bus_events_sent(&self) -> usize {
        self.bus_events_sent
    }
}
