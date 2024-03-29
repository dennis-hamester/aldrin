use crate::conn_id::ConnectionId;
use crate::core::message::Message;
#[cfg(feature = "statistics")]
use crate::BrokerStatistics;
use futures_channel::mpsc;
#[cfg(feature = "statistics")]
use futures_channel::oneshot;

#[derive(Debug)]
pub(crate) enum ConnectionEvent {
    // Sent by connections
    NewConnection(ConnectionId, mpsc::UnboundedSender<Message>),
    ConnectionShutdown(ConnectionId),
    Message(ConnectionId, Message),

    // Sent by broker handles
    ShutdownBroker,
    ShutdownIdleBroker,
    ShutdownConnection(ConnectionId),
    #[cfg(feature = "statistics")]
    TakeStatistics(oneshot::Sender<BrokerStatistics>),
}
