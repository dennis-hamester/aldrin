use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SerializeError {
    #[error("unexpected value type")]
    UnexpectedValue,

    #[error("serialized value overflowed")]
    Overflow,

    #[error("more elements serialized than expected")]
    TooManyElements,

    #[error("fewer elements serialized than expected")]
    TooFewElements,

    #[error("too deeply nested")]
    TooDeeplyNested,
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeserializeError {
    #[error("invalid serialization")]
    InvalidSerialization,

    #[error("unexpected end of input")]
    UnexpectedEoi,

    #[error("unexpected value type")]
    UnexpectedValue,

    #[error("too deeply nested")]
    TooDeeplyNested,

    #[error("no more elements")]
    NoMoreElements,

    #[error("more elements remain")]
    MoreElementsRemain,

    #[error("serialization contains trailing data")]
    TrailingData,
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error(transparent)]
pub struct ProtocolVersionError {
    #[from]
    pub(crate) kind: ProtocolVersionErrorKind,
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ProtocolVersionErrorKind {
    #[error("invalid major version component")]
    InvalidMajor,

    #[error("invalid minor version component")]
    InvalidMinor,

    #[error("version failed to parse")]
    Parse,
}

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
