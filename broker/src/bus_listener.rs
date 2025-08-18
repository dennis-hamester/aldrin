use crate::conn_id::ConnectionId;
use aldrin_core::{BusEvent, BusListenerFilter, BusListenerScope, ObjectId, ObjectUuid, ServiceId};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct BusListener {
    conn_id: ConnectionId,
    filters: HashSet<BusListenerFilter>,
    scope: Option<BusListenerScope>,
    matches_all_objects: bool,
}

impl BusListener {
    pub(crate) fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            filters: HashSet::new(),
            scope: None,
            matches_all_objects: false,
        }
    }

    pub(crate) fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub(crate) fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
        self.matches_all_objects |= filter == BusListenerFilter::Object(None);
    }

    pub(crate) fn remove_filter(&mut self, filter: BusListenerFilter) {
        self.filters.remove(&filter);

        self.matches_all_objects = self
            .filters
            .iter()
            .any(|&f| f == BusListenerFilter::Object(None));
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filters.clear();
        self.matches_all_objects = false;
    }

    pub(crate) fn start(&mut self, scope: BusListenerScope) -> bool {
        if self.scope.is_none() {
            self.scope = Some(scope);
            true
        } else {
            false
        }
    }

    pub(crate) fn stop(&mut self) -> bool {
        self.scope.take().is_some()
    }

    pub(crate) fn matches_object(&self, object: ObjectId) -> bool {
        self.matches_all_objects
            || self
                .filters
                .iter()
                .copied()
                .any(|filter| filter.matches_object(object))
    }

    pub(crate) fn matches_service(&self, service: ServiceId) -> bool {
        self.filters
            .iter()
            .copied()
            .any(|filter| filter.matches_service(service))
    }

    pub(crate) fn matches_new_event(&self, event: BusEvent) -> bool {
        self.scope
            .map(BusListenerScope::includes_new)
            .unwrap_or(false)
            && self
                .filters
                .iter()
                .copied()
                .any(|filter| filter.matches_event(event))
    }

    pub(crate) fn specific_objects(&self) -> Option<impl Iterator<Item = ObjectUuid> + '_> {
        if self.matches_all_objects {
            None
        } else {
            Some(self.filters.iter().filter_map(|f| match f {
                BusListenerFilter::Object(Some(obj)) => Some(*obj),
                BusListenerFilter::Object(None) => unreachable!(),
                BusListenerFilter::Service(_) => None,
            }))
        }
    }
}
