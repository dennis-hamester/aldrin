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
