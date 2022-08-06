use crate::conn_id::ConnectionId;
use aldrin_proto::{ObjectCookie, ServiceCookie};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Object {
    conn_id: ConnectionId,
    cookie: ObjectCookie,
    svcs: HashSet<ServiceCookie>,
}

impl Object {
    pub fn new(conn_id: ConnectionId, cookie: ObjectCookie) -> Self {
        Object {
            conn_id,
            cookie,
            svcs: HashSet::new(),
        }
    }

    pub fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub fn cookie(&self) -> ObjectCookie {
        self.cookie
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
