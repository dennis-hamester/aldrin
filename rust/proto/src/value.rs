// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::{ConversionError, FromKeyValue, IntoKeyValue, KeyValue};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hash};
use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Value {
    None,
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
    Vec(Vec<Value>),
    Map(HashMap<KeyValue, Value>),
    Set(HashSet<KeyValue>),
    Struct(HashMap<u32, Value>),
    Enum(u32, Box<Value>),
}

impl From<KeyValue> for Value {
    fn from(v: KeyValue) -> Value {
        match v {
            KeyValue::U8(v) => Value::U8(v),
            KeyValue::I8(v) => Value::I8(v),
            KeyValue::U16(v) => Value::U16(v),
            KeyValue::I16(v) => Value::I16(v),
            KeyValue::U32(v) => Value::U32(v),
            KeyValue::I32(v) => Value::I32(v),
            KeyValue::U64(v) => Value::U64(v),
            KeyValue::I64(v) => Value::I64(v),
            KeyValue::String(v) => Value::String(v),
            KeyValue::Uuid(v) => Value::Uuid(v),
        }
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

impl FromValue for u8 {
    fn from_value(v: Value) -> Result<u8, ConversionError> {
        match v {
            Value::U8(v) => Ok(v),
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
            _ => Err(ConversionError),
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
    T: FromValue,
{
    fn from_value(v: Value) -> Result<Vec<T>, ConversionError> {
        match v {
            Value::Vec(v) => v.into_iter().map(T::from_value).collect(),
            _ => Err(ConversionError),
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

#[allow(clippy::implicit_hasher)]
impl<K, V> FromValue for HashMap<K, V>
where
    K: FromKeyValue + Eq + Hash,
    V: FromValue,
{
    fn from_value(v: Value) -> Result<HashMap<K, V>, ConversionError> {
        match v {
            Value::Map(v) => v
                .into_iter()
                .map(|(k, v)| K::from_key_value(k).and_then(|k| V::from_value(v).map(|v| (k, v))))
                .collect(),
            _ => Err(ConversionError),
        }
    }
}

impl<K, V, S> IntoValue for HashMap<K, V, S>
where
    K: IntoKeyValue,
    V: IntoValue,
    S: BuildHasher,
{
    fn into_value(self) -> Value {
        Value::Map(
            self.into_iter()
                .map(|(k, v)| (k.into_key_value(), v.into_value()))
                .collect(),
        )
    }
}

#[allow(clippy::implicit_hasher)]
impl<T> FromValue for HashSet<T>
where
    T: FromKeyValue + Eq + Hash,
{
    fn from_value(v: Value) -> Result<HashSet<T>, ConversionError> {
        match v {
            Value::Set(v) => v.into_iter().map(T::from_key_value).collect(),
            _ => Err(ConversionError),
        }
    }
}

impl<T, S> IntoValue for HashSet<T, S>
where
    T: IntoKeyValue,
    S: BuildHasher,
{
    fn into_value(self) -> Value {
        Value::Set(self.into_iter().map(T::into_key_value).collect())
    }
}
