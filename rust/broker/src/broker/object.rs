use super::ServiceCookie;
use crate::conn_id::ConnectionId;
use std::collections::HashSet;
use uuid::Uuid;

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

    pub fn services<'a>(&'a self) -> impl Iterator<Item = ServiceCookie> + 'a {
        self.svcs.iter().copied()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ObjectUuid(pub Uuid);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ObjectCookie(pub Uuid);
