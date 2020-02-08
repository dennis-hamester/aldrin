use aldrin_proto::Message;

#[derive(Debug, Clone)]
pub(crate) enum BrokerEvent {
    Shutdown,
    Message(Message),
}
