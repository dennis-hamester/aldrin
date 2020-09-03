use crate::conn_id::ConnectionId;

/// Handle to a specific connection.
///
/// `ConnectionHandle`s can be acquired with [`Connection::handle`](crate::Connection::handle) and
/// used to [shut down](crate::BrokerHandle::shutdown_connection) the
/// [`Connection`](crate::Connection).
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    id: ConnectionId,
}

impl ConnectionHandle {
    pub(super) fn new(id: ConnectionId) -> Self {
        ConnectionHandle { id }
    }

    pub(crate) fn id(&self) -> &ConnectionId {
        &self.id
    }

    pub(super) fn into_id(self) -> ConnectionId {
        self.id
    }
}
