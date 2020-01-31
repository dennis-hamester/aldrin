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

use super::ConversionError;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    String(String),
    Uuid(Uuid),
}

pub trait FromKeyValue: Sized {
    fn from_key_value(v: KeyValue) -> Result<Self, ConversionError>;
}

pub trait IntoKeyValue {
    fn into_key_value(self) -> KeyValue;
}

impl FromKeyValue for KeyValue {
    fn from_key_value(v: KeyValue) -> Result<KeyValue, ConversionError> {
        Ok(v)
    }
}

impl IntoKeyValue for KeyValue {
    fn into_key_value(self) -> KeyValue {
        self
    }
}

impl FromKeyValue for u8 {
    fn from_key_value(v: KeyValue) -> Result<u8, ConversionError> {
        match v {
            KeyValue::U8(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for u8 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::U8(self)
    }
}

impl FromKeyValue for i8 {
    fn from_key_value(v: KeyValue) -> Result<i8, ConversionError> {
        match v {
            KeyValue::I8(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for i8 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::I8(self)
    }
}

impl FromKeyValue for u16 {
    fn from_key_value(v: KeyValue) -> Result<u16, ConversionError> {
        match v {
            KeyValue::U16(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for u16 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::U16(self)
    }
}

impl FromKeyValue for i16 {
    fn from_key_value(v: KeyValue) -> Result<i16, ConversionError> {
        match v {
            KeyValue::I16(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for i16 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::I16(self)
    }
}

impl FromKeyValue for u32 {
    fn from_key_value(v: KeyValue) -> Result<u32, ConversionError> {
        match v {
            KeyValue::U32(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for u32 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::U32(self)
    }
}

impl FromKeyValue for i32 {
    fn from_key_value(v: KeyValue) -> Result<i32, ConversionError> {
        match v {
            KeyValue::I32(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for i32 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::I32(self)
    }
}

impl FromKeyValue for u64 {
    fn from_key_value(v: KeyValue) -> Result<u64, ConversionError> {
        match v {
            KeyValue::U64(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for u64 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::U64(self)
    }
}

impl FromKeyValue for i64 {
    fn from_key_value(v: KeyValue) -> Result<i64, ConversionError> {
        match v {
            KeyValue::I64(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for i64 {
    fn into_key_value(self) -> KeyValue {
        KeyValue::I64(self)
    }
}

impl FromKeyValue for String {
    fn from_key_value(v: KeyValue) -> Result<String, ConversionError> {
        match v {
            KeyValue::String(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for String {
    fn into_key_value(self) -> KeyValue {
        KeyValue::String(self)
    }
}

impl IntoKeyValue for &str {
    fn into_key_value(self) -> KeyValue {
        KeyValue::String(self.to_owned())
    }
}

impl FromKeyValue for Uuid {
    fn from_key_value(v: KeyValue) -> Result<Uuid, ConversionError> {
        match v {
            KeyValue::Uuid(v) => Ok(v),
            _ => Err(ConversionError),
        }
    }
}

impl IntoKeyValue for Uuid {
    fn into_key_value(self) -> KeyValue {
        KeyValue::Uuid(self)
    }
}
