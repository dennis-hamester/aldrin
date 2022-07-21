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

    /// The number of current connections.
    pub num_connections: usize,

    /// The number of new connections added.
    pub connections_added: usize,

    /// The number of connections shut down.
    pub connections_shut_down: usize,
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
            num_connections: 0,
            connections_added: 0,
            connections_shut_down: 0,
        }
    }

    pub(super) fn take(&mut self) -> Self {
        let now = Instant::now();
        let mut res = self.clone();

        // Fixup timestamps.
        res.end = now;
        self.start = now;

        // Reset statistics to 0.
        self.connections_added = 0;
        self.connections_shut_down = 0;

        res
    }
}
