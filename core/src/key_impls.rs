use crate::tags::{self, KeyTag, KeyTagImpl, PrimaryKeyTag};
use crate::{DeserializeError, DeserializeKey, SerializeError, SerializeKey};
use std::borrow::Cow;
use uuid::Uuid;

macro_rules! impl_primitive {
    { $ty:ty $( :primary $primary:ty )? } => {
        impl_primitive! {
            $ty $( :primary $primary )?
            :tag tags::U8, tags::I8, tags::U16, tags::I16,
                 tags::U32, tags::I32, tags::U64, tags::I64
        }
    };

    { $ty:ty $( :primary $primary:ty )? :tag $( $tag:ty ),+ } => {
        $(
            impl PrimaryKeyTag for $ty {
                type KeyTag = $primary;
            }
        )?

        $(
            impl SerializeKey<$tag> for $ty {
                fn try_as_key(&self) -> Result<<$tag as KeyTagImpl>::Key<'_>, SerializeError> {
                    (*self).try_into().map_err(|_| SerializeError::UnexpectedValue)
                }
            }

            impl DeserializeKey<$tag> for $ty {
                fn try_from_key(key: <$tag as KeyTagImpl>::Key<'_>,) -> Result<Self, DeserializeError> {
                    key.try_into().map_err(|_| DeserializeError::UnexpectedValue)
                }
            }
        )+
    };
}

impl_primitive!(u8 :primary tags::U8);
impl_primitive!(i8 :primary tags::I8);
impl_primitive!(u16 :primary tags::U16);
impl_primitive!(i16 :primary tags::I16);
impl_primitive!(u32 :primary tags::U32);
impl_primitive!(i32 :primary tags::I32);
impl_primitive!(u64 :primary tags::U64);
impl_primitive!(i64 :primary tags::I64);
impl_primitive!(usize :primary tags::U64);
impl_primitive!(isize :primary tags::I64);
impl_primitive!(u128);
impl_primitive!(i128);

impl PrimaryKeyTag for String {
    type KeyTag = tags::String;
}

impl SerializeKey<tags::String> for String {
    fn try_as_key(&self) -> Result<Cow<str>, SerializeError> {
        Ok(Cow::Borrowed(self))
    }
}

impl DeserializeKey<tags::String> for String {
    fn try_from_key(key: Cow<str>) -> Result<Self, DeserializeError> {
        Ok(key.into_owned())
    }
}

impl PrimaryKeyTag for str {
    type KeyTag = tags::String;
}

impl SerializeKey<tags::String> for str {
    fn try_as_key(&self) -> Result<Cow<Self>, SerializeError> {
        Ok(Cow::Borrowed(self))
    }
}

impl PrimaryKeyTag for Uuid {
    type KeyTag = tags::Uuid;
}

impl SerializeKey<tags::Uuid> for Uuid {
    fn try_as_key(&self) -> Result<Self, SerializeError> {
        Ok(*self)
    }
}

impl DeserializeKey<tags::Uuid> for Uuid {
    fn try_from_key(key: Self) -> Result<Self, DeserializeError> {
        Ok(key)
    }
}

impl<T: PrimaryKeyTag + ?Sized> PrimaryKeyTag for &T {
    type KeyTag = T::KeyTag;
}

impl<T: KeyTag, U: SerializeKey<T> + ?Sized> SerializeKey<T> for &U {
    fn try_as_key(&self) -> Result<<T::Impl as KeyTagImpl>::Key<'_>, SerializeError> {
        (**self).try_as_key()
    }
}

impl<T: PrimaryKeyTag + ?Sized> PrimaryKeyTag for &mut T {
    type KeyTag = T::KeyTag;
}

impl<T: KeyTag, U: SerializeKey<T> + ?Sized> SerializeKey<T> for &mut U {
    fn try_as_key(&self) -> Result<<T::Impl as KeyTagImpl>::Key<'_>, SerializeError> {
        (**self).try_as_key()
    }
}

impl<T: PrimaryKeyTag + ?Sized> PrimaryKeyTag for Box<T> {
    type KeyTag = T::KeyTag;
}

impl<T: KeyTag, U: SerializeKey<T> + ?Sized> SerializeKey<T> for Box<U> {
    fn try_as_key(&self) -> Result<<T::Impl as KeyTagImpl>::Key<'_>, SerializeError> {
        (**self).try_as_key()
    }
}

impl<T: KeyTag, U: DeserializeKey<T>> DeserializeKey<T> for Box<U> {
    fn try_from_key(key: <T::Impl as KeyTagImpl>::Key<'_>) -> Result<Self, DeserializeError> {
        U::try_from_key(key).map(Self::new)
    }
}
