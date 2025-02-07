use crate::{
    tags, DeserializeError, Serialize, SerializedValue, SerializedValueSlice, Struct, ValueKind,
};
use std::collections::hash_map::{HashMap, IntoIter, Iter};
use std::convert::Infallible;
use std::iter::{self, Empty, Map};
use std::ops::{Deref, DerefMut};

pub trait AsUnknownFields {
    type Field: Serialize<tags::Value>;
    type FieldsIter: ExactSizeIterator<Item = (u32, Self::Field)>;

    fn fields(self) -> Self::FieldsIter;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnknownFields(pub HashMap<u32, SerializedValue>);

impl UnknownFields {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_fields_set(&self) -> bool {
        self.0
            .values()
            .filter_map(|val| val.kind().ok())
            .any(|kind| kind != ValueKind::None)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (u32, &SerializedValueSlice)> {
        self.into_iter()
    }

    pub fn deserialize_as_value(&self) -> Result<Struct, DeserializeError> {
        self.iter()
            .map(|(id, val)| val.deserialize_as_value().map(|val| (id, val)))
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

impl AsUnknownFields for UnknownFields {
    type Field = SerializedValue;
    type FieldsIter = <Self as IntoIterator>::IntoIter;

    fn fields(self) -> Self::FieldsIter {
        self.into_iter()
    }
}

impl<'a> AsUnknownFields for &'a UnknownFields {
    type Field = &'a SerializedValueSlice;
    type FieldsIter = <Self as IntoIterator>::IntoIter;

    fn fields(self) -> Self::FieldsIter {
        self.into_iter()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UnknownFieldsRef<Fields>(pub Fields);

impl<Fields, Field> AsUnknownFields for UnknownFieldsRef<Fields>
where
    Fields: IntoIterator<Item = (u32, Field)>,
    Fields::IntoIter: ExactSizeIterator,
    Field: Serialize<tags::Value>,
{
    type Field = Field;
    type FieldsIter = Fields::IntoIter;

    fn fields(self) -> Self::FieldsIter {
        self.0.into_iter()
    }
}

impl AsUnknownFields for () {
    type Field = Infallible;
    type FieldsIter = Empty<(u32, Infallible)>;

    fn fields(self) -> Self::FieldsIter {
        iter::empty()
    }
}

impl AsUnknownFields for Infallible {
    type Field = Self;
    type FieldsIter = Empty<(u32, Self)>;

    fn fields(self) -> Self::FieldsIter {
        iter::empty()
    }
}
