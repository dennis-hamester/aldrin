#[cfg(feature = "introspection")]
use crate::introspection::{ArrayType, BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use std::collections::{LinkedList, VecDeque};
use std::mem::MaybeUninit;

macro_rules! impl_vec {
    { $ty:ident } => {
        impl<T: PrimaryTag> PrimaryTag for $ty<T> {
            type Tag = tags::Vec<T::Tag>;
        }

        impl<T: Tag, U: Serialize<T>> Serialize<tags::Vec<T>> for $ty<U> {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }

        impl<'a, T, U> Serialize<tags::Vec<T>> for &'a $ty<U>
        where
            T: Tag,
            &'a U: Serialize<T>,
        {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }

        impl<T: Tag, U: Deserialize<T>> Deserialize<tags::Vec<T>> for $ty<U> {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                deserializer.deserialize_vec_extend_new()
            }
        }

        #[cfg(feature = "introspection")]
        impl<T: Introspectable> Introspectable for $ty<T> {
            fn layout() -> Layout {
                BuiltInType::Vec(T::lexical_id()).into()
            }

            fn lexical_id() -> LexicalId {
                LexicalId::vec(T::lexical_id())
            }

            fn add_references(references: &mut References) {
                references.add::<T>();
            }
        }
    }
}

impl_vec!(Vec);
impl_vec!(VecDeque);
impl_vec!(LinkedList);

impl<T: PrimaryTag> PrimaryTag for &[T] {
    type Tag = tags::Vec<T::Tag>;
}

impl<'a, T, U> Serialize<tags::Vec<T>> for &'a [U]
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for [T] {
    fn layout() -> Layout {
        BuiltInType::Vec(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::vec(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

impl<T: PrimaryTag, const N: usize> PrimaryTag for [T; N] {
    type Tag = tags::Vec<T::Tag>;
}

impl<T: Tag, U: Serialize<T>, const N: usize> Serialize<tags::Vec<T>> for [U; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<'a, T, U, const N: usize> Serialize<tags::Vec<T>> for &'a [U; N]
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Tag, U: Deserialize<T>, const N: usize> Deserialize<tags::Vec<T>> for [U; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_vec()?;

        // SAFETY: This creates an array of MaybeUninit<U>, which doesn't require initialization.
        let mut arr: [MaybeUninit<U>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut num = 0;

        for elem in &mut arr {
            match deserializer.deserialize() {
                Ok(value) => {
                    elem.write(value);
                    num += 1;
                }

                Err(e) => {
                    for elem in &mut arr[..num] {
                        // SAFETY: The first num elements have been initialized.
                        unsafe {
                            elem.assume_init_drop();
                        }
                    }

                    return Err(e);
                }
            }
        }

        // SAFETY: Exactly num elements have been initialized and num equals N.
        //
        // It's currently impossible to transmute [MaybeUninit<U>; N] to [U; N] when U is a generic
        // or N a const generic. See https://github.com/rust-lang/rust/issues/61956.
        let value = unsafe {
            (*(&MaybeUninit::new(arr) as *const _ as *const MaybeUninit<[U; N]>)).assume_init_read()
        };

        deserializer.finish(value)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable, const N: usize> Introspectable for [T; N] {
    fn layout() -> Layout {
        BuiltInType::Array(ArrayType::new(T::lexical_id(), N as u32)).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::array(T::lexical_id(), N as u32)
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

impl<T> Serialize<tags::Vec<T>> for bytes::Bytes
where
    T: Tag,
    u8: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T> Serialize<tags::Vec<T>> for &bytes::Bytes
where
    T: Tag,
    u8: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self.iter().copied())
    }
}

impl<T> Deserialize<tags::Vec<T>> for bytes::Bytes
where
    T: Tag,
    u8: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize_vec_extend_new::<T, u8, Vec<u8>>()
            .map(Self::from)
    }
}

impl<T> Serialize<tags::Vec<T>> for bytes::BytesMut
where
    T: Tag,
    u8: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T> Serialize<tags::Vec<T>> for &bytes::BytesMut
where
    T: Tag,
    u8: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self.iter().copied())
    }
}

impl<T> Deserialize<tags::Vec<T>> for bytes::BytesMut
where
    T: Tag,
    u8: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec_extend_new::<T, u8, _>()
    }
}

impl<T: Tag> Serialize<tags::Vec<T>> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec(0)?.finish()
    }
}

impl<T: Tag> Serialize<tags::Vec<T>> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec(0)?.finish()
    }
}

impl<T: Tag> Deserialize<tags::Vec<T>> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec()?.finish(())
    }
}
