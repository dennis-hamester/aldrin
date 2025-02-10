use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Tag, Value,
};
use std::collections::{LinkedList, VecDeque};
use std::mem::MaybeUninit;

macro_rules! impl_vec {
    { $ty:ident } => {
        impl<T: PrimaryTag> PrimaryTag for $ty<T> {
            type Tag = Vec<T::Tag>;
        }

        impl<T: Tag, U: Serialize<T>> Serialize<Vec<T>> for $ty<U> {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }

        impl<T: Tag, U: Deserialize<T>> Deserialize<Vec<T>> for $ty<U> {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                deserializer.deserialize_vec_extend_new()
            }
        }

        impl<'a, T, U> Serialize<Vec<T>> for &'a $ty<U>
        where
            T: Tag,
            &'a U: Serialize<T>,
        {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }

        impl<T: Serialize<Value>> Serialize<Value> for $ty<T> {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }

        impl<T: Deserialize<Value>> Deserialize<Value> for $ty<T> {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                deserializer.deserialize_vec_extend_new()
            }
        }

        impl<'a, T> Serialize<Value> for &'a $ty<T>
        where
            &'a T: Serialize<Value>,
        {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self)
            }
        }
    }
}

impl_vec!(Vec);
impl_vec!(VecDeque);
impl_vec!(LinkedList);

impl<T: PrimaryTag> PrimaryTag for &[T] {
    type Tag = Vec<T::Tag>;
}

impl<'a, T, U> Serialize<Vec<T>> for &'a [U]
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<'a, T> Serialize<Value> for &'a [T]
where
    &'a T: Serialize<Value>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: PrimaryTag, const N: usize> PrimaryTag for [T; N] {
    type Tag = Vec<T::Tag>;
}

impl<T: Tag, U: Serialize<T>, const N: usize> Serialize<Vec<T>> for [U; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Tag, U: Deserialize<T>, const N: usize> Deserialize<Vec<T>> for [U; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_vec()?;

        if deserializer.len() != N {
            return Err(DeserializeError::UnexpectedValue);
        }

        // SAFETY: This creates an array of MaybeUninit<U>, which doesn't require initialization.
        let mut arr: [MaybeUninit<U>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Manually count the number of elements, so that the safety of this function doesn't depend
        // on the correctness of VecDeserializer.
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

        debug_assert_eq!(num, N);

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

impl<'a, T, U, const N: usize> Serialize<Vec<T>> for &'a [U; N]
where
    T: Tag,
    &'a U: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Serialize<Value>, const N: usize> Serialize<Value> for [T; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}

impl<T: Deserialize<Value>, const N: usize> Deserialize<Value> for [T; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Vec<Value>, _>()
    }
}

impl<'a, T, const N: usize> Serialize<Value> for &'a [T; N]
where
    &'a T: Serialize<Value>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_vec_iter(self)
    }
}
