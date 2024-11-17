use crate::conn_id::ConnectionId;
use crate::core::message::{QueryIntrospectionReply, QueryIntrospectionResult};
use crate::core::{SerializedValue, TypeId};
use rand::Rng;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::mem;

#[derive(Debug)]
pub(crate) struct IntrospectionDatabase {
    entries: HashMap<TypeId, IntrospectionEntry>,
}

impl IntrospectionDatabase {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[cfg(feature = "statistics")]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn register(&mut self, type_ids: &HashSet<TypeId>, conn_id: &ConnectionId) {
        for type_id in type_ids {
            self.entries
                .entry(*type_id)
                .or_default()
                .register(conn_id.clone());
        }
    }

    pub fn remove_conn(&mut self, conn_id: &ConnectionId) -> Vec<RemoveConn> {
        let mut result = Vec::new();

        self.entries.retain(|&type_id, entry| {
            let was_queried = entry.queried();
            let retain = entry.remove_conn(conn_id);
            let is_queried = entry.queried();

            if let (Some(serial), None) = (was_queried, is_queried) {
                if retain {
                    result.push(RemoveConn::cont(serial, type_id));
                } else {
                    result.push(RemoveConn::unavailable(serial, entry.take_pending()));
                }
            }

            retain
        });

        result
    }

    pub fn get_mut(&mut self, type_id: TypeId) -> Option<&mut IntrospectionEntry> {
        self.entries.get_mut(&type_id)
    }

    pub fn query_replied(
        &mut self,
        type_id: TypeId,
        conn_id: &ConnectionId,
        reply: QueryIntrospectionReply,
    ) -> Option<IntrospectionQueryResult> {
        let Entry::Occupied(mut entry) = self.entries.entry(type_id) else {
            panic!("inconsistent state");
        };

        if !entry.get_mut().query_replied(conn_id, reply.serial) {
            return None;
        }

        match reply.result {
            QueryIntrospectionResult::Ok(introspection) => {
                let entry = entry.into_mut();
                let pending = entry.take_pending();
                let introspection = entry.set_introspection(introspection);

                Some(IntrospectionQueryResult::Available {
                    introspection,
                    pending,
                })
            }

            QueryIntrospectionResult::Unavailable => {
                if entry.get_mut().remove_conn(conn_id) {
                    Some(IntrospectionQueryResult::Continue(entry.into_mut()))
                } else {
                    let pending = entry.remove().take_pending();
                    Some(IntrospectionQueryResult::Unavailable(pending))
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct IntrospectionEntry {
    conn_id_idxs: HashMap<ConnectionId, usize>,
    conn_ids: Vec<ConnectionId>,
    introspection: Option<SerializedValue>,
    queried: Option<IntrospectionQuery>,
    pending: Vec<IntrospectionQuery>,
}

impl IntrospectionEntry {
    fn register(&mut self, conn_id: ConnectionId) {
        if let Entry::Vacant(idx_entry) = self.conn_id_idxs.entry(conn_id.clone()) {
            let idx = self.conn_ids.len();
            self.conn_ids.push(conn_id);
            idx_entry.insert(idx);
        }
    }

    fn remove_conn(&mut self, conn_id: &ConnectionId) -> bool {
        if let Some(ref queried) = self.queried {
            if queried.conn_id == *conn_id {
                self.queried = None;
            }
        }

        self.pending.retain(|pending| pending.conn_id != *conn_id);

        if let Some(idx) = self.conn_id_idxs.remove(conn_id) {
            if self.conn_id_idxs.is_empty() {
                false
            } else {
                self.conn_ids.swap_remove(idx);

                if idx != self.conn_ids.len() {
                    let conn_id = &self.conn_ids[idx];

                    *self
                        .conn_id_idxs
                        .get_mut(conn_id)
                        .expect("inconsistent state") = idx;
                }

                true
            }
        } else {
            debug_assert!(!self.conn_id_idxs.is_empty());
            debug_assert!(!self.conn_ids.is_empty());

            true
        }
    }

    pub fn introspection(&self) -> Option<&SerializedValue> {
        self.introspection.as_ref()
    }

    pub fn queried(&self) -> Option<u32> {
        self.queried.as_ref().map(|queried| queried.serial)
    }

    pub fn add_pending(&mut self, conn_id: ConnectionId, serial: u32) {
        debug_assert!(self.introspection.is_none());
        self.pending.push(IntrospectionQuery::new(conn_id, serial));
    }

    pub fn query_random_conn(&mut self, serial: u32) -> &ConnectionId {
        debug_assert!(self.queried.is_none());
        debug_assert!(!self.conn_id_idxs.is_empty());
        debug_assert!(!self.conn_ids.is_empty());

        let idx = rand::thread_rng().gen_range(0..self.conn_ids.len());
        let conn_id = &self.conn_ids[idx];

        self.queried = Some(IntrospectionQuery::new(conn_id.clone(), serial));
        conn_id
    }

    fn query_replied(&mut self, conn_id: &ConnectionId, serial: u32) -> bool {
        if let Some(ref queried) = self.queried {
            if queried.conn_id == *conn_id {
                debug_assert_eq!(serial, queried.serial);
                debug_assert!(self.introspection.is_none());

                self.queried = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn set_introspection(&mut self, introspection: SerializedValue) -> &SerializedValue {
        debug_assert!(self.introspection.is_none());
        self.introspection.insert(introspection)
    }

    fn take_pending(&mut self) -> Vec<IntrospectionQuery> {
        mem::take(&mut self.pending)
    }
}

#[derive(Debug)]
pub(crate) struct IntrospectionQuery {
    pub conn_id: ConnectionId,
    pub serial: u32,
}

impl IntrospectionQuery {
    fn new(conn_id: ConnectionId, serial: u32) -> Self {
        Self { conn_id, serial }
    }
}

#[derive(Debug)]
pub(crate) enum IntrospectionQueryResult<'a> {
    Available {
        introspection: &'a SerializedValue,
        pending: Vec<IntrospectionQuery>,
    },

    Unavailable(Vec<IntrospectionQuery>),
    Continue(&'a mut IntrospectionEntry),
}

#[derive(Debug)]
pub(crate) enum RemoveConnResult {
    Unavailable(Vec<IntrospectionQuery>),
    Continue(TypeId),
}

#[derive(Debug)]
pub(crate) struct RemoveConn {
    pub serial: u32,
    pub result: RemoveConnResult,
}

impl RemoveConn {
    fn unavailable(serial: u32, pending: Vec<IntrospectionQuery>) -> Self {
        Self {
            serial,
            result: RemoveConnResult::Unavailable(pending),
        }
    }

    fn cont(serial: u32, type_id: TypeId) -> Self {
        Self {
            serial,
            result: RemoveConnResult::Continue(type_id),
        }
    }
}
