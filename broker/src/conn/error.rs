use crate::core::message::Message;
use crate::core::{DeserializeError, SerializeError};
use thiserror::Error;

/// Error of an active connection.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError<T> {
    /// The broker shut down unexpectedly.
    #[error("broker shut down unexpectedly")]
    UnexpectedShutdown,

    /// The transport encountered an error.
    #[error(transparent)]
    Transport(#[from] T),
}

/// Error while establishing a new connection.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EstablishError<T> {
    /// An unexpected message was received.
    #[error("unexpected message received")]
    UnexpectedMessageReceived(Message),

    /// The protocol version of the client is incompatible.
    #[error("incompatible protocol version")]
    IncompatibleVersion,

    /// The broker shut down.
    #[error("broker shut down")]
    Shutdown,

    /// The transport encountered an error.
    #[error(transparent)]
    Transport(T),

    /// A value failed to serialize.
    #[error(transparent)]
    Serialize(#[from] SerializeError),

    /// A value failed to deserialize.
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
}
