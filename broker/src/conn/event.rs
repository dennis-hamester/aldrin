use crate::conn_id::ConnectionId;
use crate::versioned_message::VersionedMessage;
#[cfg(feature = "statistics")]
use crate::BrokerStatistics;
use aldrin_core::message::Message;
use aldrin_core::ProtocolVersion;
use futures_channel::mpsc;
#[cfg(feature = "statistics")]
use futures_channel::oneshot;

#[derive(Debug)]
pub(crate) enum ConnectionEvent {
    // Sent by connections
    NewConnection(
        ConnectionId,
        ProtocolVersion,
        mpsc::UnboundedSender<VersionedMessage>,
    ),

    ConnectionShutdown(ConnectionId),
    Message(ConnectionId, Message),

    // Sent by broker handles
    ShutdownBroker,
    ShutdownIdleBroker,
    ShutdownConnection(ConnectionId),

    #[cfg(feature = "statistics")]
    TakeStatistics(oneshot::Sender<BrokerStatistics>),
}
