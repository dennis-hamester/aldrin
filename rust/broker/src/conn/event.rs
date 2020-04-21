use crate::conn_id::ConnectionId;
use aldrin_proto::Message;
use futures_channel::mpsc::Sender;

#[derive(Debug)]
pub(crate) enum ConnectionEvent {
    // Sent by connections
    NewConnection(ConnectionId, Sender<Message>),
    ConnectionShutdown(ConnectionId),
    Message(ConnectionId, Message),

    // Sent by broker handles
    ShutdownBroker,
    ShutdownIdleBroker,
    ShutdownConnection(ConnectionId),
}
