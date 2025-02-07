use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use std::fmt;
use std::marker::PhantomData;

pub struct AsValue<T, U> {
    pub inner: U,
    tag: PhantomData<T>,
}

impl<T, U> AsValue<T, U> {
    pub fn new(inner: U) -> Self {
        Self {
            inner,
            tag: PhantomData,
        }
    }

    pub fn into_inner(self) -> U {
        self.inner
    }
}

impl<T, U> PrimaryTag for AsValue<T, U> {
    type Tag = tags::Value;
}

impl<T: Tag, U: Serialize<T>> Serialize<tags::Value> for AsValue<T, U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(self.inner)
    }
}

impl<'a, T, U> Serialize<tags::Value> for &'a AsValue<T, U>
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self.inner)
    }
}

impl<T, U: fmt::Debug> fmt::Debug for AsValue<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AsValue")
            .field("inner", &self.inner)
            .finish()
    }
}
