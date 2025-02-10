use crate::{Bytes, KeyTag, ObjectId, ServiceId, Value};
use std::marker::PhantomData;
use uuid::Uuid;

pub trait Tag: Sized {}

impl Tag for () {}

impl Tag for Value {}

impl<T: Tag> Tag for Option<T> {}

impl Tag for bool {}

impl Tag for u8 {}

impl Tag for i8 {}

impl Tag for u16 {}

impl Tag for i16 {}

impl Tag for u32 {}

impl Tag for i32 {}

impl Tag for u64 {}

impl Tag for i64 {}

impl Tag for f32 {}

impl Tag for f64 {}

impl Tag for String {}

impl Tag for Uuid {}

impl Tag for ObjectId {}

impl Tag for ServiceId {}

impl<T: Tag> Tag for Vec<T> {}

impl Tag for Bytes {}

#[derive(Debug)]
pub struct Map<K, T>(PhantomData<(K, T)>);

impl<K: KeyTag, T: Tag> Tag for Map<K, T> {}

#[derive(Debug)]
pub struct Set<T>(PhantomData<T>);

impl<T: KeyTag> Tag for Set<T> {}

#[derive(Debug)]
pub struct Sender<T>(PhantomData<T>);

impl<T: Tag> Tag for Sender<T> {}

#[derive(Debug)]
pub struct Receiver<T>(PhantomData<T>);

impl<T: Tag> Tag for Receiver<T> {}
