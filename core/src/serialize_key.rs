use crate::KeyTag;
use std::borrow::Cow;
use uuid::Uuid;

pub trait SerializeKey<T: KeyTag> {
    fn as_key(&self) -> T::Key<'_>;
}

macro_rules! impl_serialize_key {
    { $ty:ty $( , $other:ty )* } => {
        impl SerializeKey<$ty> for $ty {
            fn as_key(&self) -> <$ty as KeyTag>::Key<'_> {
                *self
            }
        }

        $(
            impl SerializeKey<$ty> for $other {
                fn as_key(&self) -> <$ty as KeyTag>::Key<'_> {
                    (*self).into()
                }
            }
        )*
    };
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

impl_serialize_key!(u8);
impl_serialize_key!(i8);
impl_serialize_key!(u16, u8);
impl_serialize_key!(i16, u8, i8);
impl_serialize_key!(u32, u8, u16);
impl_serialize_key!(i32, u8, i8, u16, i16);
impl_serialize_key!(u64, u8, u16, u32);
impl_serialize_key!(i64, u8, i8, u16, i16, u32, i32);
impl_serialize_key!(Uuid);

impl SerializeKey<Self> for String {
    fn as_key(&self) -> <Self as KeyTag>::Key<'_> {
        Cow::Borrowed(self)
    }
}

impl SerializeKey<String> for str {
    fn as_key(&self) -> <String as KeyTag>::Key<'_> {
        Cow::Borrowed(self)
    }
}
