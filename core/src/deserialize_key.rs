use crate::tags::{KeyTag, KeyTagImpl};
use crate::DeserializeError;

pub trait DeserializeKey<T: KeyTag>: Sized {
    fn try_from_key(key: <T::Impl as KeyTagImpl>::Key<'_>) -> Result<Self, DeserializeError>;
}
