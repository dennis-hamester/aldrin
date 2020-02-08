use aldrin_proto::Message;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ConnectionError {
    InternalError,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionError::InternalError => f.write_str("internal error"),
        }
    }
}

impl Error for ConnectionError {}

#[derive(Debug, Clone)]
pub enum EstablishError {
    InternalError,
    UnexpectedClientShutdown,
    UnexpectedMessageReceived(Message),
    VersionMismatch(u32),
    BrokerShutdown,
}

impl fmt::Display for EstablishError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EstablishError::InternalError => f.write_str("internal error"),
            EstablishError::UnexpectedClientShutdown => f.write_str("unexpected client shutdown"),
            EstablishError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
            EstablishError::VersionMismatch(v) => {
                f.write_fmt(format_args!("client version {} mismatch", v))
            }
            EstablishError::BrokerShutdown => f.write_str("broker shutdown"),
        }
    }
}

impl Error for EstablishError {}
