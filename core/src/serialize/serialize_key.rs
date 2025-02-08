use crate::{tag, KeyTag};
use std::borrow::Cow;
use uuid::Uuid;

pub trait SerializeKey<T: KeyTag> {
    fn as_key(&self) -> T::Key<'_>;
}

impl<T: KeyTag, U: SerializeKey<T> + ?Sized> SerializeKey<T> for &U {
    fn as_key(&self) -> T::Key<'_> {
        (**self).as_key()
    }
}

impl<T: KeyTag, U: SerializeKey<T> + ?Sized> SerializeKey<T> for Box<U> {
    fn as_key(&self) -> T::Key<'_> {
        (**self).as_key()
    }
}

macro_rules! impl_serialize_key {
    { $ty:ty, $tag:ty $( , $other:ty )* } => {
        impl SerializeKey<$tag> for $ty {
            fn as_key(&self) -> <$tag as KeyTag>::Key<'_> {
                *self
            }
        }

        $(
            impl SerializeKey<$tag> for $other {
                fn as_key(&self) -> <$tag as KeyTag>::Key<'_> {
                    (*self).into()
                }
            }
        )*
    }
}

impl_serialize_key!(u8, tag::U8);
impl_serialize_key!(i8, tag::I8);
impl_serialize_key!(u16, tag::U16, u8);
impl_serialize_key!(i16, tag::I16, u8, i8);
impl_serialize_key!(u32, tag::U32, u8, u16);
impl_serialize_key!(i32, tag::I32, u8, i8, u16, i16);
impl_serialize_key!(u64, tag::U64, u8, u16, u32);
impl_serialize_key!(i64, tag::I64, u8, i8, u16, i16, u32, i32);
impl_serialize_key!(Uuid, tag::Uuid);

impl SerializeKey<tag::String> for String {
    fn as_key(&self) -> <tag::String as KeyTag>::Key<'_> {
        Cow::Borrowed(self)
    }
}

impl SerializeKey<tag::String> for str {
    fn as_key(&self) -> <tag::String as KeyTag>::Key<'_> {
        Cow::Borrowed(self)
    }
}
