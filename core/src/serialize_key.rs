use crate::tags::{KeyTag, KeyTagImpl};
use crate::SerializeError;

pub trait SerializeKey<T: KeyTag> {
    fn try_as_key(&self) -> Result<<T::Impl as KeyTagImpl>::Key<'_>, SerializeError>;
}
