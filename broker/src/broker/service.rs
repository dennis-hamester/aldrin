use super::ConnectionId;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Service {
    function_calls: HashSet<u32>,

    /// Map of events subscribed by a set of connections.
    events: HashMap<u32, HashSet<ConnectionId>>,

    /// Set of connections subscribed to all events.
    all_events: HashSet<ConnectionId>,

    /// Set of connections subscribed to this service.
    subscriptions: HashSet<ConnectionId>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            function_calls: HashSet::new(),
            events: HashMap::new(),
            all_events: HashSet::new(),
            subscriptions: HashSet::new(),
        }
    }

    pub fn add_function_call(&mut self, serial: u32) {
        let unique = self.function_calls.insert(serial);
        debug_assert!(unique);
    }

    pub fn remove_function_call(&mut self, serial: u32) {
        let contained = self.function_calls.remove(&serial);
        debug_assert!(contained);
    }

    pub fn function_calls(&self) -> impl Iterator<Item = u32> + '_ {
        self.function_calls.iter().copied()
    }

    pub fn subscribe_event(&mut self, event: u32, conn_id: ConnectionId) -> bool {
        match self.events.entry(event) {
            Entry::Occupied(mut subs) => {
                subs.get_mut().insert(conn_id);
                false
            }

            Entry::Vacant(subs) => {
                subs.insert(HashSet::with_capacity(1)).insert(conn_id);
                true
            }
        }
    }

    /// Removes subscription of `event` made by connection `conn_id`.
    ///
    /// Returns `true` if a subscription was removed *and* it was the last one of `event`, `false`
    /// otherwise.
    pub fn unsubscribe_event(&mut self, event: u32, conn_id: &ConnectionId) -> bool {
        match self.events.entry(event) {
            Entry::Occupied(mut subs) => {
                subs.get_mut().remove(conn_id);
                if subs.get().is_empty() {
                    subs.remove();
                    true
                } else {
                    false
                }
            }

            Entry::Vacant(_) => false,
        }
    }

    pub fn subscribe_all_events(&mut self, conn_id: ConnectionId) -> bool {
        let was_empty = self.all_events.is_empty();
        self.all_events.insert(conn_id);
        was_empty
    }

    pub fn unsubscribe_all_events(&mut self, conn_id: &ConnectionId) -> bool {
        let was_empty = self.all_events.is_empty();
        self.all_events.remove(conn_id);
        !was_empty && self.all_events.is_empty()
    }

    pub fn subscribe(&mut self, conn_id: ConnectionId) {
        self.subscriptions.insert(conn_id);
    }

    pub fn unsubscribe(&mut self, conn_id: &ConnectionId) {
        self.subscriptions.remove(conn_id);
    }

    pub fn subscribed_conn_ids(&self) -> impl Iterator<Item = &ConnectionId> {
        #[allow(clippy::mutable_key_type)]
        let mut res = HashSet::new();

        res.extend(self.events.values().flatten());
        res.extend(self.subscriptions.iter());

        res.into_iter()
    }
}
