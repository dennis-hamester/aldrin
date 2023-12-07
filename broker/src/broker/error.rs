use thiserror::Error;

/// Broker has shut down.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("broker shut down")]
pub struct BrokerShutdown;
