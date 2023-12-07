use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SerializeError {
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

    #[error("no more elements")]
    NoMoreElements,

    #[error("too deeply nested")]
    TooDeeplyNested,

    #[error("serialization contains trailing data")]
    TrailingData,
}
