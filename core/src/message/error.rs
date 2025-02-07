use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageDeserializeError {
    #[error("invalid serialization")]
    InvalidSerialization,

    #[error("unexpected end of input")]
    UnexpectedEoi,

    #[error("unexpected message type")]
    UnexpectedMessage,

    #[error("serialization contains trailing data")]
    TrailingData,
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageSerializeError {
    #[error("serialized message overflowed")]
    Overflow,

    #[error("invalid value")]
    InvalidValue,
}
