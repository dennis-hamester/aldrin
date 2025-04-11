use super::{
    Bool, Bytes, Infallible, KeyTag, Map, ObjectId, Option, Receiver, Sender, ServiceId, Set,
    String, Unit, Uuid, Value, Vec, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8,
};

pub trait Tag: Sized {}

pub trait PrimaryTag {
    type Tag: Tag;
}

pub type As<T> = <T as PrimaryTag>::Tag;

impl Tag for Value {}

impl Tag for Unit {}

impl<T: Tag> Tag for Option<T> {}

impl Tag for Bool {}

impl Tag for U8 {}

impl Tag for I8 {}

impl Tag for U16 {}

impl Tag for I16 {}

impl Tag for U32 {}

impl Tag for I32 {}

impl Tag for U64 {}

impl Tag for I64 {}

impl Tag for F32 {}

impl Tag for F64 {}

impl Tag for String {}

impl Tag for Uuid {}

impl Tag for ObjectId {}

impl Tag for ServiceId {}

impl<T: Tag> Tag for Vec<T> {}

impl Tag for Bytes {}

impl<K: KeyTag, T: Tag> Tag for Map<K, T> {}

impl<T: KeyTag> Tag for Set<T> {}

impl<T: Tag> Tag for Sender<T> {}

impl<T: Tag> Tag for Receiver<T> {}

impl Tag for Infallible {}
