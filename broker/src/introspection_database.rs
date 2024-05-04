use crate::conn_id::ConnectionId;
use crate::core::TypeId;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct IntrospectionDatabase {
    entries: HashMap<TypeId, DatabaseEntry>,
}

impl IntrospectionDatabase {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, type_ids: &HashSet<TypeId>, conn_id: &ConnectionId) {
        for type_id in type_ids {
            self.entries
                .entry(*type_id)
                .or_default()
                .conn_ids
                .insert(conn_id.clone());
        }
    }

    pub fn remove_conn(&mut self, conn_id: &ConnectionId) {
        self.entries.retain(|_, entry| {
            entry.conn_ids.remove(conn_id);
            !entry.conn_ids.is_empty()
        });
    }
}

#[derive(Debug, Default)]
struct DatabaseEntry {
    conn_ids: HashSet<ConnectionId>,
}
