use crate::conn_id::ConnectionId;

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
