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

pub const VERSION: u32 = 1;

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
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    SubscribeObjectsCreated(SubscribeObjectsCreated),
    SubscribeObjectsCreatedReply(SubscribeObjectsCreatedReply),
    UnsubscribeObjectsCreated,
    ObjectCreatedEvent(ObjectCreatedEvent),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    SubscribeObjectsDestroyed,
    UnsubscribeObjectsDestroyed,
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    SubscribeServicesCreated(SubscribeServicesCreated),
    SubscribeServicesCreatedReply(SubscribeServicesCreatedReply),
    UnsubscribeServicesCreated,
    ServiceCreatedEvent(ServiceCreatedEvent),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServicesDestroyed,
    UnsubscribeServicesDestroyed,
    ServiceDestroyedEvent(ServiceDestroyedEvent),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
}

#[derive(Debug, Clone)]
pub struct Connect {
    pub version: u32,
}

#[derive(Debug, Clone)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[derive(Debug, Clone)]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: Uuid,
}

#[derive(Debug, Clone)]
pub enum CreateObjectResult {
    Ok(Uuid),
    DuplicateObject,
}

#[derive(Debug, Clone)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

#[derive(Debug, Clone)]
pub struct SubscribeObjectsCreated {
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SubscribeObjectsCreatedReply {
    pub serial: u32,
}

#[derive(Debug, Clone)]
pub struct ObjectCreatedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: Uuid,
}

#[derive(Debug, Clone)]
pub enum DestroyObjectResult {
    Ok,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug, Clone)]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

#[derive(Debug, Clone)]
pub struct ObjectDestroyedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[derive(Debug, Clone)]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
}

#[derive(Debug, Clone)]
pub enum CreateServiceResult {
    Ok(Uuid),
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug, Clone)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[derive(Debug, Clone)]
pub struct SubscribeServicesCreated {
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SubscribeServicesCreatedReply {
    pub serial: u32,
}

#[derive(Debug, Clone)]
pub struct ServiceCreatedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: Uuid,
}

#[derive(Debug, Clone)]
pub enum DestroyServiceResult {
    Ok,
    InvalidService,
    ForeignObject,
}

#[derive(Debug, Clone)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[derive(Debug, Clone)]
pub struct ServiceDestroyedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[derive(Debug, Clone)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: Uuid,
    pub function: u32,
    pub args: Value,
}

#[derive(Debug, Clone)]
pub enum CallFunctionResult {
    Ok(Value),
    Err(Value),
    Aborted,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

#[derive(Debug, Clone)]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}
