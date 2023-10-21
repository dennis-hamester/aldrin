use crate::conn_id::ConnectionId;
use aldrin_proto::ServiceCookie;
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Object {
    conn_id: ConnectionId,
    svcs: HashSet<ServiceCookie>,
}

impl Object {
    pub fn new(conn_id: ConnectionId) -> Self {
        Object {
            conn_id,
            svcs: HashSet::new(),
        }
    }

    pub fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub fn add_service(&mut self, cookie: ServiceCookie) {
        let unique = self.svcs.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_service(&mut self, cookie: ServiceCookie) {
        let contained = self.svcs.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn services(&self) -> impl Iterator<Item = ServiceCookie> + '_ {
        self.svcs.iter().copied()
    }
}
