use aldrin_proto::Message;
use std::error::Error;
use std::fmt;

/// Error of an active connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError<T> {
    /// The broker shut down unexpectedly.
    UnexpectedBrokerShutdown,

    /// The transport encountered an error.
    Transport(T),
}

impl<T> From<T> for ConnectionError<T> {
    fn from(e: T) -> Self {
        ConnectionError::Transport(e)
    }
}

impl<T> fmt::Display for ConnectionError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionError::UnexpectedBrokerShutdown => f.write_str("unexpected broker shutdown"),
            ConnectionError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T> Error for ConnectionError<T> where T: fmt::Debug + fmt::Display {}

/// Error while establishing a new connection.
#[derive(Debug, Clone)]
pub enum EstablishError<T> {
    /// An unexpected message was received.
    UnexpectedMessageReceived(Message),

    /// Protocol version mismatch between broker and client.
    VersionMismatch(u32),

    /// The broker shut down.
    BrokerShutdown,

    /// The transport encountered an error.
    Transport(T),
}

impl<T> From<T> for EstablishError<T> {
    fn from(e: T) -> Self {
        EstablishError::Transport(e)
    }
}

impl<T> fmt::Display for EstablishError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EstablishError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
            EstablishError::VersionMismatch(v) => {
                f.write_fmt(format_args!("client version {} mismatch", v))
            }
            EstablishError::BrokerShutdown => f.write_str("broker shutdown"),
            EstablishError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T> Error for EstablishError<T> where T: fmt::Debug + fmt::Display {}
