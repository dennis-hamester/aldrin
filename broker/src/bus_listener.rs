use crate::conn_id::ConnectionId;
use aldrin_proto::BusListenerFilter;
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct BusListener {
    conn_id: ConnectionId,
    filters: HashSet<BusListenerFilter>,
}

impl BusListener {
    pub fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            filters: HashSet::new(),
        }
    }

    pub fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }

    pub fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
    }
}
