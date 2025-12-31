use super::{I8, I16, I32, I64, KeyTagImpl, String, U8, U16, U32, U64, Uuid};

pub trait KeyTag: Sized {
    type Impl: KeyTagImpl;
}

pub trait PrimaryKeyTag {
    type KeyTag: KeyTag;
}

pub type AsKey<T> = <T as PrimaryKeyTag>::KeyTag;

impl KeyTag for U8 {
    type Impl = Self;
}

impl KeyTag for I8 {
    type Impl = Self;
}

impl KeyTag for U16 {
    type Impl = Self;
}

impl KeyTag for I16 {
    type Impl = Self;
}

impl KeyTag for U32 {
    type Impl = Self;
}

impl KeyTag for I32 {
    type Impl = Self;
}

impl KeyTag for U64 {
    type Impl = Self;
}

impl KeyTag for I64 {
    type Impl = Self;
}

impl KeyTag for String {
    type Impl = Self;
}

impl KeyTag for Uuid {
    type Impl = Self;
}
