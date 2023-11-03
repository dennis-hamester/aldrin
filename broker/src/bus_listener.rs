use crate::conn_id::ConnectionId;
use aldrin_proto::{BusListenerFilter, BusListenerScope, ObjectId, ServiceId};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct BusListener {
    conn_id: ConnectionId,
    filters: HashSet<BusListenerFilter>,
    scope: Option<BusListenerScope>,
}

impl BusListener {
    pub fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            filters: HashSet::new(),
            scope: None,
        }
    }

    pub fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
    }

    pub fn remove_filter(&mut self, filter: BusListenerFilter) {
        self.filters.remove(&filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub fn start(&mut self, scope: BusListenerScope) -> bool {
        match self.scope {
            Some(_) => false,

            None => {
                self.scope = Some(scope);
                true
            }
        }
    }

    pub fn stop(&mut self) -> bool {
        self.scope.take().is_some()
    }

    pub fn matches_object(&self, object: ObjectId) -> bool {
        self.filters
            .iter()
            .copied()
            .any(|filter| filter.matches_object(object))
    }

    pub fn matches_service(&self, service: ServiceId) -> bool {
        self.filters
            .iter()
            .copied()
            .any(|filter| filter.matches_service(service))
    }
}
