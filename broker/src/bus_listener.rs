use crate::conn_id::ConnectionId;

#[derive(Debug)]
pub(crate) struct BusListener {
    conn_id: ConnectionId,
}

impl BusListener {
    pub fn new(conn_id: ConnectionId) -> Self {
        Self { conn_id }
    }

    pub fn conn_id(&self) -> &ConnectionId {
        &self.conn_id
    }
}
