use std::error::Error;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SerializeError {
    Overflow,
    TooManyElements,
    TooFewElements,
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("serialized value overflowed"),
            Self::TooManyElements => f.write_str("more elements serialized than expected"),
            Self::TooFewElements => f.write_str("less elements serialized than expected"),
        }
    }
}

impl Error for SerializeError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeserializeError {
    InvalidSerialization,
    UnexpectedEoi,
    UnexpectedValue,
    NoMoreElements,
    TrailingData,
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidSerialization => f.write_str("invalid serialization"),
            Self::UnexpectedEoi => f.write_str("unexpected end of input"),
            Self::UnexpectedValue => f.write_str("unexpected value type"),
            Self::NoMoreElements => f.write_str("no more elements"),
            Self::TrailingData => f.write_str("serialization contains trailing data"),
        }
    }
}

impl Error for DeserializeError {}
