mod bytes;
mod infallible;
mod map;
mod option;
mod primitive;
mod result;
mod set;
mod string;
#[cfg(test)]
mod test;
mod tuple;
mod unit;
mod uuid;
mod vec;

#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use std::borrow::Cow;

impl<T: PrimaryTag + ?Sized> PrimaryTag for &T {
    type Tag = T::Tag;
}

impl<T: PrimaryTag + ?Sized> PrimaryTag for &mut T {
    type Tag = T::Tag;
}

impl<'a, T, U> Serialize<T> for &&'a U
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (*self).serialize(serializer)
    }
}

impl<'a, T, U> Serialize<T> for &'a mut U
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (&*self).serialize(serializer)
    }
}

impl<'a, T, U> Serialize<T> for &'a &mut U
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (&**self).serialize(serializer)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for &T {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for &mut T {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

impl<T: PrimaryTag + ?Sized> PrimaryTag for Box<T> {
    type Tag = T::Tag;
}

impl<T, U> Serialize<T> for Box<U>
where
    T: Tag,
    U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (*self).serialize(serializer)
    }
}

impl<'a, T, U> Serialize<T> for &'a Box<U>
where
    T: Tag,
    U: ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        (**self).serialize(serializer)
    }
}

impl<T, U> Deserialize<T> for Box<U>
where
    T: Tag,
    U: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        U::deserialize(deserializer).map(Self::new)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for Box<T> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Box(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::box_ty(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

impl<'a, T> PrimaryTag for Cow<'a, T>
where
    T: 'a + ToOwned + PrimaryTag + ?Sized,
{
    type Tag = T::Tag;
}

impl<'a, T, U> Serialize<T> for Cow<'a, U>
where
    T: Tag,
    U: 'a + ToOwned + ?Sized,
    for<'b> &'b U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<T>(self.as_ref())
    }
}

impl<'a, 'b, T, U> Serialize<T> for &'a Cow<'b, U>
where
    T: Tag,
    U: 'b + ToOwned + ?Sized,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<T>(self.as_ref())
    }
}

impl<'a, T, U> Deserialize<T> for Cow<'a, U>
where
    T: Tag,
    U: 'a + ToOwned + ?Sized,
    U::Owned: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<T, _>().map(Self::Owned)
    }
}
