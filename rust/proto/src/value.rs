#[cfg(test)]
mod test;

use crate::{ObjectId, ServiceId};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::hash::BuildHasher;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum Value {
    None,
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Uuid(Uuid),
    ObjectId(ObjectId),
    ServiceId(ServiceId),
    Vec(Vec<Value>),
    Bytes(Vec<u8>),
    U8Map(HashMap<u8, Value>),
    I8Map(HashMap<i8, Value>),
    U16Map(HashMap<u16, Value>),
    I16Map(HashMap<i16, Value>),
    U32Map(HashMap<u32, Value>),
    I32Map(HashMap<i32, Value>),
    U64Map(HashMap<u64, Value>),
    I64Map(HashMap<i64, Value>),
    StringMap(HashMap<String, Value>),
    UuidMap(HashMap<Uuid, Value>),
    U8Set(HashSet<u8>),
    I8Set(HashSet<i8>),
    U16Set(HashSet<u16>),
    I16Set(HashSet<i16>),
    U32Set(HashSet<u32>),
    I32Set(HashSet<i32>),
    U64Set(HashSet<u64>),
    I64Set(HashSet<i64>),
    StringSet(HashSet<String>),
    UuidSet(HashSet<Uuid>),
    Struct(HashMap<u32, Value>),
    Enum { variant: u32, value: Box<Value> },
}

impl Value {
    // Converts this value to another type.
    //
    // `convert` can be used with any type `T` that implements the [`FromValue`] trait.
    pub fn convert<T: FromValue>(self) -> Result<T, ConversionError> {
        T::from_value(self)
    }
}

/// Wrapper for `Vec<u8>`.
///
/// This wrapper exists only to enable different implementations of [`FromValue`] and [`IntoValue`]
/// than those for `Vec<u8>`, which convert between `u8` and [`Value`].
#[derive(Debug, Clone)]
pub struct Bytes(pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait FromValue: Sized {
    fn from_value(v: Value) -> Result<Self, ConversionError>;
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl FromValue for Value {
    fn from_value(v: Value) -> Result<Value, ConversionError> {
        Ok(v)
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl FromValue for () {
    fn from_value(v: Value) -> Result<(), ConversionError> {
        match v {
            Value::None => Ok(()),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for () {
    fn into_value(self) -> Value {
        Value::None
    }
}

impl<T> FromValue for Box<T>
where
    T: FromValue,
{
    fn from_value(v: Value) -> Result<Box<T>, ConversionError> {
        Ok(Box::new(T::from_value(v)?))
    }
}

impl<T> IntoValue for Box<T>
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        (*self).into_value()
    }
}

impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn from_value(v: Value) -> Result<Option<T>, ConversionError> {
        match v {
            Value::None => Ok(None),
            v => T::from_value(v).map(Some),
        }
    }
}

impl<T> IntoValue for Option<T>
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        match self {
            Some(v) => v.into_value(),
            None => Value::None,
        }
    }
}

impl FromValue for bool {
    fn from_value(v: Value) -> Result<bool, ConversionError> {
        match v {
            Value::Bool(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl FromValue for u8 {
    fn from_value(v: Value) -> Result<u8, ConversionError> {
        match v {
            Value::U8(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for u8 {
    fn into_value(self) -> Value {
        Value::U8(self)
    }
}

impl FromValue for i8 {
    fn from_value(v: Value) -> Result<i8, ConversionError> {
        match v {
            Value::I8(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for i8 {
    fn into_value(self) -> Value {
        Value::I8(self)
    }
}

impl FromValue for u16 {
    fn from_value(v: Value) -> Result<u16, ConversionError> {
        match v {
            Value::U16(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for u16 {
    fn into_value(self) -> Value {
        Value::U16(self)
    }
}

impl FromValue for i16 {
    fn from_value(v: Value) -> Result<i16, ConversionError> {
        match v {
            Value::I16(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for i16 {
    fn into_value(self) -> Value {
        Value::I16(self)
    }
}

impl FromValue for u32 {
    fn from_value(v: Value) -> Result<u32, ConversionError> {
        match v {
            Value::U32(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for u32 {
    fn into_value(self) -> Value {
        Value::U32(self)
    }
}

impl FromValue for i32 {
    fn from_value(v: Value) -> Result<i32, ConversionError> {
        match v {
            Value::I32(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::I32(self)
    }
}

impl FromValue for u64 {
    fn from_value(v: Value) -> Result<u64, ConversionError> {
        match v {
            Value::U64(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for u64 {
    fn into_value(self) -> Value {
        Value::U64(self)
    }
}

impl FromValue for i64 {
    fn from_value(v: Value) -> Result<i64, ConversionError> {
        match v {
            Value::I64(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::I64(self)
    }
}

impl FromValue for f32 {
    fn from_value(v: Value) -> Result<f32, ConversionError> {
        match v {
            Value::F32(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::F32(self)
    }
}

impl FromValue for f64 {
    fn from_value(v: Value) -> Result<f64, ConversionError> {
        match v {
            Value::F64(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::F64(self)
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<String, ConversionError> {
        match v {
            Value::String(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::String(self.to_owned())
    }
}

impl FromValue for Uuid {
    fn from_value(v: Value) -> Result<Uuid, ConversionError> {
        match v {
            Value::Uuid(v) => Ok(v),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for Uuid {
    fn into_value(self) -> Value {
        Value::Uuid(self)
    }
}

impl<T> FromValue for Vec<T>
where
    T: FromValue + IntoValue,
{
    fn from_value(v: Value) -> Result<Vec<T>, ConversionError> {
        match v {
            Value::Vec(v) => {
                let len = v.len();
                let mut res = Vec::with_capacity(len);

                let mut iter = v.into_iter();
                for v in &mut iter {
                    match T::from_value(v) {
                        Ok(v) => res.push(v),
                        Err(e) => {
                            // Restore original value
                            let mut err = Vec::with_capacity(len);
                            err.extend(res.into_iter().map(T::into_value));
                            if let Some(v) = e.0 {
                                err.push(v);
                            }
                            err.extend(iter);
                            return Err(ConversionError(Some(Value::Vec(err))));
                        }
                    }
                }

                Ok(res)
            }

            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl<T> IntoValue for Vec<T>
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        Value::Vec(self.into_iter().map(T::into_value).collect())
    }
}

impl FromValue for Bytes {
    fn from_value(v: Value) -> Result<Self, ConversionError> {
        match v {
            Value::Bytes(v) => Ok(Bytes(v)),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

impl IntoValue for Bytes {
    fn into_value(self) -> Value {
        Value::Bytes(self.0)
    }
}

macro_rules! impl_map {
    ($key:ty, $var:ident) => {
        impl<V, S> FromValue for HashMap<$key, V, S>
        where
            V: FromValue + IntoValue,
            S: BuildHasher + Default,
        {
            fn from_value(v: Value) -> Result<HashMap<$key, V, S>, ConversionError> {
                match v {
                    Value::$var(v) => {
                        let len = v.len();
                        let mut res = HashMap::with_capacity_and_hasher(v.len(), S::default());

                        let mut iter = v.into_iter();
                        for (k, v) in &mut iter {
                            match V::from_value(v) {
                                Ok(v) => {
                                    res.insert(k, v);
                                }

                                Err(e) => {
                                    // Restore original value
                                    let mut err = HashMap::with_capacity(len);
                                    err.extend(res.into_iter().map(|(k, v)| (k, v.into_value())));
                                    if let Some(v) = e.0 {
                                        err.insert(k, v);
                                    }
                                    err.extend(iter);
                                    return Err(ConversionError(Some(Value::$var(err))));
                                }
                            }
                        }

                        Ok(res)
                    }

                    _ => Err(ConversionError(Some(v))),
                }
            }
        }

        impl<V, S> IntoValue for HashMap<$key, V, S>
        where
            V: IntoValue,
        {
            fn into_value(self) -> Value {
                Value::$var(self.into_iter().map(|(k, v)| (k, v.into_value())).collect())
            }
        }
    };
}

impl_map!(u8, U8Map);
impl_map!(i8, I8Map);
impl_map!(u16, U16Map);
impl_map!(i16, I16Map);
impl_map!(u32, U32Map);
impl_map!(i32, I32Map);
impl_map!(u64, U64Map);
impl_map!(i64, I64Map);
impl_map!(String, StringMap);
impl_map!(Uuid, UuidMap);

macro_rules! impl_set {
    // Implement these only for the default BuildHasher, because then they are zero-copy. If you
    // need implementations for other BuildHashers, then please open a ticket.
    ($key:ty, $var:ident) => {
        #[allow(clippy::implicit_hasher)]
        impl FromValue for HashSet<$key> {
            fn from_value(v: Value) -> Result<HashSet<$key>, ConversionError> {
                match v {
                    Value::$var(v) => Ok(v),
                    _ => Err(ConversionError(Some(v))),
                }
            }
        }

        #[allow(clippy::implicit_hasher)]
        impl IntoValue for HashSet<$key> {
            fn into_value(self) -> Value {
                Value::$var(self)
            }
        }
    };
}

impl_set!(u8, U8Set);
impl_set!(i8, I8Set);
impl_set!(u16, U16Set);
impl_set!(i16, I16Set);
impl_set!(u32, U32Set);
impl_set!(i32, I32Set);
impl_set!(u64, U64Set);
impl_set!(i64, I64Set);
impl_set!(String, StringSet);
impl_set!(Uuid, UuidSet);

impl IntoValue for Infallible {
    fn into_value(self) -> Value {
        match self {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionError(pub Option<Value>);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid conversion")
    }
}

impl Error for ConversionError {}
