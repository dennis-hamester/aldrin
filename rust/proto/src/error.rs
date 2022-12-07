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
pub struct DeserializeError;

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("value deserialization failed")
    }
}

impl Error for DeserializeError {}
