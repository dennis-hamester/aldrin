use std::borrow::Cow;
use uuid::Uuid;

pub trait Sealed {}

pub trait KeyTag: Sealed + Sized {
    type Key<'a>;
}

macro_rules! impl_key_tag {
    { $tag:ty } => {
        impl Sealed for $tag {}

        impl KeyTag for $tag {
            type Key<'a> = Self;
        }
    };
}

impl_key_tag!(u8);
impl_key_tag!(i8);
impl_key_tag!(u16);
impl_key_tag!(i16);
impl_key_tag!(u32);
impl_key_tag!(i32);
impl_key_tag!(u64);
impl_key_tag!(i64);

impl Sealed for String {}

impl KeyTag for String {
    type Key<'a> = Cow<'a, str>;
}

impl Sealed for Uuid {}

impl KeyTag for Uuid {
    type Key<'a> = Self;
}
