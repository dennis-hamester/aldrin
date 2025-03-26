use crate::tags::Tag;
use crate::Serializer;
use thiserror::Error;

pub trait Serialize<T: Tag>: Sized {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError>;
}

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
