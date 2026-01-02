use crate::conn_id::ConnectionId;
use aldrin_core::{
    BusEvent, BusListenerFilter, BusListenerScope, BusListenerServiceFilter, ObjectId, ObjectUuid,
    ServiceId, ServiceUuid,
};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct BusListener {
    conn_id: ConnectionId,
    filters: HashSet<BusListenerFilter>,
    scope: Option<BusListenerScope>,
    matches_all_objects: bool,
    matches_specific_services: bool,
}

impl BusListener {
    pub(crate) fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            filters: HashSet::new(),
            scope: None,
            matches_all_objects: false,
            matches_specific_services: true,
        }
    }

    pub(crate) fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub(crate) fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
        self.matches_all_objects |= filter == BusListenerFilter::Object(None);

        self.matches_specific_services &= matches!(
            filter,
            BusListenerFilter::Service(BusListenerServiceFilter {
                object: Some(_),
                service: Some(_),
            })
        );
    }

    pub(crate) fn remove_filter(&mut self, filter: BusListenerFilter) {
        self.filters.remove(&filter);

        self.matches_all_objects = self
            .filters
            .iter()
            .any(|&f| f == BusListenerFilter::Object(None));

        self.matches_specific_services = self.filters.iter().all(|f| {
            matches!(
                f,
                BusListenerFilter::Service(BusListenerServiceFilter {
                    object: Some(_),
                    service: Some(_),
                })
            )
        });
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filters.clear();
        self.matches_all_objects = false;
        self.matches_specific_services = true;
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
        self.scope.is_some_and(BusListenerScope::includes_new)
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

    pub(crate) fn specific_services(
        &self,
    ) -> Option<impl Iterator<Item = (ObjectUuid, ServiceUuid)> + '_> {
        if self.matches_specific_services {
            Some(self.filters.iter().filter_map(|f| match f {
                BusListenerFilter::Service(BusListenerServiceFilter {
                    object: Some(obj),
                    service: Some(svc),
                }) => Some((*obj, *svc)),

                BusListenerFilter::Object(_) => None,
                BusListenerFilter::Service(_) => unreachable!(),
            }))
        } else {
            None
        }
    }
}
