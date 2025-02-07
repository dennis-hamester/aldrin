use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Enum, Serialize, SerializedValue, SerializedValueSlice,
};
use std::convert::Infallible;

pub trait AsUnknownVariant {
    type Value: Serialize<tags::Value>;

    fn id(&self) -> u32;
    fn value(self) -> Self::Value;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVariant {
    id: u32,
    value: SerializedValue,
}

impl UnknownVariant {
    pub fn new(id: u32, value: SerializedValue) -> Self {
        Self { id, value }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn value(&self) -> &SerializedValueSlice {
        &self.value
    }

    pub fn into_value(self) -> SerializedValue {
        self.value
    }

    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        self.value.deserialize_as()
    }

    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    pub fn deserialize_as_value(&self) -> Result<Enum, DeserializeError> {
        self.deserialize()
    }
}

impl AsUnknownVariant for UnknownVariant {
    type Value = SerializedValue;

    fn id(&self) -> u32 {
        self.id
    }

    fn value(self) -> Self::Value {
        self.value
    }
}

impl<'a> AsUnknownVariant for &'a UnknownVariant {
    type Value = &'a SerializedValueSlice;

    fn id(&self) -> u32 {
        self.id
    }

    fn value(self) -> Self::Value {
        &self.value
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UnknownVariantRef<T> {
    pub id: u32,
    pub value: T,
}

impl<T> UnknownVariantRef<T> {
    pub fn new(id: u32, value: T) -> Self {
        Self { id, value }
    }
}

impl<T: Serialize<tags::Value>> AsUnknownVariant for UnknownVariantRef<T> {
    type Value = T;

    fn id(&self) -> u32 {
        self.id
    }

    fn value(self) -> Self::Value {
        self.value
    }
}

impl AsUnknownVariant for Infallible {
    type Value = Self;

    fn id(&self) -> u32 {
        match *self {}
    }

    fn value(self) -> Self::Value {
        self
    }
}
