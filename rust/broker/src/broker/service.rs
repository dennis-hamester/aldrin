use super::ConnectionId;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct Service {
    function_calls: HashSet<u32>,

    /// Map of active subscriptions made by other connection
    subscriptions: HashMap<u32, HashSet<ConnectionId>>,
}

impl Service {
    pub fn new() -> Self {
        Service {
            function_calls: HashSet::new(),
            subscriptions: HashMap::new(),
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

    pub fn function_calls<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.function_calls.iter().copied()
    }

    pub fn add_subscription(&mut self, id: u32, conn_id: ConnectionId) -> bool {
        match self.subscriptions.entry(id) {
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

    /// Removes subscription of event `id` made by connection `conn_id`.
    ///
    /// Returns `true` if a subscription was removed *and* it was the last one of event `id`,
    /// `false` otherwise.
    pub fn remove_subscription(&mut self, id: u32, conn_id: &ConnectionId) -> bool {
        match self.subscriptions.entry(id) {
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

    pub fn subscribed_conn_ids(&self) -> impl Iterator<Item = &ConnectionId> {
        let mut res = HashSet::new();
        for conn_ids in self.subscriptions.values() {
            res.extend(conn_ids);
        }
        res.into_iter()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ServiceUuid(pub Uuid);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ServiceCookie(pub Uuid);
