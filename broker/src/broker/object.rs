use crate::conn_id::ConnectionId;
use aldrin_core::ServiceCookie;
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Object {
    conn_id: ConnectionId,
    svcs: HashSet<ServiceCookie>,
}

impl Object {
    pub(crate) fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            svcs: HashSet::new(),
        }
    }

    pub(crate) fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub(crate) fn add_service(&mut self, cookie: ServiceCookie) {
        let unique = self.svcs.insert(cookie);
        debug_assert!(unique);
    }

    pub(crate) fn remove_service(&mut self, cookie: ServiceCookie) {
        let contained = self.svcs.remove(&cookie);
        debug_assert!(contained);
    }

    pub(crate) fn services(&self) -> impl Iterator<Item = ServiceCookie> + '_ {
        self.svcs.iter().copied()
    }
}
