use aldrin_proto::Message;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ConnectionError<T> {
    InternalError,
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
            ConnectionError::InternalError => f.write_str("internal error"),
            ConnectionError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T> Error for ConnectionError<T> where T: fmt::Debug + fmt::Display {}

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
