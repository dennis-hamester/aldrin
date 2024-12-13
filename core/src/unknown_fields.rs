use crate::error::DeserializeError;
use crate::generic_value::Struct;
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use std::collections::hash_map::{HashMap, IntoIter, Iter};
use std::iter::Map;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnknownFields(pub HashMap<u32, SerializedValue>);

impl UnknownFields {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (u32, &SerializedValueSlice)> {
        self.into_iter()
    }

    pub fn deserialize_as_value(&self) -> Result<Struct, DeserializeError> {
        self.iter()
            .map(|(id, val)| val.deserialize().map(|val| (id, val)))
            .collect::<Result<_, _>>()
            .map(Struct)
    }
}

impl Deref for UnknownFields {
    type Target = HashMap<u32, SerializedValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnknownFields {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for UnknownFields {
    type Item = (u32, SerializedValue);
    type IntoIter = IntoIter<u32, SerializedValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a UnknownFields {
    type Item = (u32, &'a SerializedValueSlice);
    type IntoIter = Map<
        Iter<'a, u32, SerializedValue>,
        fn((&'a u32, &'a SerializedValue)) -> (u32, &'a SerializedValueSlice),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(id, val)| (*id, val))
    }
}
