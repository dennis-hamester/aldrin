use crate::tag;
use std::borrow::Cow;
use uuid::Uuid;

pub trait Sealed {}

pub trait KeyTag: Sealed + Sized {
    type Key<'a>;
}

macro_rules! impl_key_tag {
    { $tag:ty, $key:ty } => {
        impl Sealed for $tag {}

        impl KeyTag for $tag {
            type Key<'a> = $key;
        }
    }
}

impl_key_tag!(tag::U8, u8);
impl_key_tag!(tag::I8, i8);
impl_key_tag!(tag::U16, u16);
impl_key_tag!(tag::I16, i16);
impl_key_tag!(tag::U32, u32);
impl_key_tag!(tag::I32, i32);
impl_key_tag!(tag::U64, u64);
impl_key_tag!(tag::I64, i64);

impl Sealed for tag::String {}

impl KeyTag for tag::String {
    type Key<'a> = Cow<'a, str>;
}

impl Sealed for tag::Uuid {}

impl KeyTag for tag::Uuid {
    type Key<'a> = Uuid;
}
