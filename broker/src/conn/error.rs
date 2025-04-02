use aldrin_core::{DeserializeError, SerializeError, ValueConversionError};
use thiserror::Error;

/// Error of an active connection.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError<T> {
    /// The broker shut down unexpectedly.
    #[error("broker shut down unexpectedly")]
    UnexpectedShutdown,

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

impl<T> From<ValueConversionError> for ConnectionError<T> {
    fn from(err: ValueConversionError) -> Self {
        match err {
            // Conversion here is always passed a valid version.
            ValueConversionError::InvalidVersion => unreachable!(),

            ValueConversionError::Serialize(e) => Self::Serialize(e),
            ValueConversionError::Deserialize(e) => Self::Deserialize(e),
        }
    }
}
