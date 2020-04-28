use crate::conn_id::ConnectionId;
use aldrin_proto::Message;
use futures_channel::mpsc::UnboundedSender;

#[derive(Debug)]
pub(crate) enum ConnectionEvent {
    // Sent by connections
    NewConnection(ConnectionId, UnboundedSender<Message>),
    ConnectionShutdown(ConnectionId),
    Message(ConnectionId, Message),

    // Sent by broker handles
    ShutdownBroker,
    ShutdownIdleBroker,
    ShutdownConnection(ConnectionId),
}
