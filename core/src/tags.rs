mod base;
mod key;
mod key_impl;

use std::marker::PhantomData;

pub use base::{As, PrimaryTag, Tag};
pub use key::{AsKey, KeyTag, PrimaryKeyTag};
pub use key_impl::KeyTagImpl;

#[derive(Debug)]
pub struct Value(());

#[derive(Debug)]
pub struct Unit(());

#[derive(Debug)]
pub struct Option<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Bool(());

#[derive(Debug)]
pub struct U8(());

#[derive(Debug)]
pub struct I8(());

#[derive(Debug)]
pub struct U16(());

#[derive(Debug)]
pub struct I16(());

#[derive(Debug)]
pub struct U32(());

#[derive(Debug)]
pub struct I32(());

#[derive(Debug)]
pub struct U64(());

#[derive(Debug)]
pub struct I64(());

#[derive(Debug)]
pub struct F32(());

#[derive(Debug)]
pub struct F64(());

#[derive(Debug)]
pub struct String(());

#[derive(Debug)]
pub struct Uuid(());

#[derive(Debug)]
pub struct ObjectId(());

#[derive(Debug)]
pub struct ServiceId(());

#[derive(Debug)]
pub struct Vec<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Bytes(());

#[derive(Debug)]
pub struct Map<K, T>(PhantomData<(K, T)>);

#[derive(Debug)]
pub struct Set<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Sender<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Receiver<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Infallible(());
