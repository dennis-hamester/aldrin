use std::marker::PhantomData;

pub trait Tag: Sized {}

#[derive(Debug)]
#[non_exhaustive]
pub struct Unit;

impl Tag for Unit {}

#[derive(Debug)]
pub struct Option<T>(PhantomData<T>);

impl<T: Tag> Tag for Option<T> {}

#[derive(Debug)]
#[non_exhaustive]
pub struct Bool;

impl Tag for Bool {}

#[derive(Debug)]
#[non_exhaustive]
pub struct U8;

impl Tag for U8 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct I8;

impl Tag for I8 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct U16;

impl Tag for U16 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct I16;

impl Tag for I16 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct U32;

impl Tag for U32 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct I32;

impl Tag for I32 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct U64;

impl Tag for U64 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct I64;

impl Tag for I64 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct F32;

impl Tag for F32 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct F64;

impl Tag for F64 {}

#[derive(Debug)]
#[non_exhaustive]
pub struct String;

impl Tag for String {}

#[derive(Debug)]
#[non_exhaustive]
pub struct Uuid;

impl Tag for Uuid {}

#[derive(Debug)]
#[non_exhaustive]
pub struct ObjectId;

impl Tag for ObjectId {}

#[derive(Debug)]
#[non_exhaustive]
pub struct ServiceId;

impl Tag for ServiceId {}

#[derive(Debug)]
pub struct Vec<T>(PhantomData<T>);

impl<T: Tag> Tag for Vec<T> {}

#[derive(Debug)]
#[non_exhaustive]
pub struct Bytes;

impl Tag for Bytes {}

#[derive(Debug)]
pub struct Map<K, T>(PhantomData<(K, T)>);

impl<K, T: Tag> Tag for Map<K, T> {}

#[derive(Debug)]
pub struct Set<T>(PhantomData<T>);

impl<T> Tag for Set<T> {}

#[derive(Debug)]
pub struct Sender<T>(PhantomData<T>);

impl<T: Tag> Tag for Sender<T> {}

#[derive(Debug)]
pub struct Receiver<T>(PhantomData<T>);

impl<T: Tag> Tag for Receiver<T> {}
