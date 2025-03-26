use crate::tags::Tag;
use crate::Deserializer;
use thiserror::Error;

pub trait Deserialize<T: Tag>: Sized {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError>;
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
