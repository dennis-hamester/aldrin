// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
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

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct CallFunction {
    pub serial: u32,
    pub object_id: Uuid,
    pub service_id: Uuid,
    pub function: u32,
    pub args: Value,
}

#[derive(Debug, Clone)]
pub enum CallFunctionResult {
    Ok(Value),
    Err(Value),
    InvalidObject,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

#[derive(Debug, Clone)]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}
