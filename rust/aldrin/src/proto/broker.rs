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

use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum BrokerMessage {
    ConnectReply(ConnectReply),
    CreateObjectReply(CreateObjectReply),
    DestroyObjectReply(DestroyObjectReply),
    ObjectCreatedEvent(ObjectCreatedEvent),
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateServiceReply(CreateServiceReply),
    DestroyServiceReply(DestroyServiceReply),
    ServiceCreatedEvent(ServiceCreatedEvent),
    ServiceDestroyedEvent(ServiceDestroyedEvent),
}

#[derive(Debug, Clone)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[derive(Debug, Clone)]
pub enum CreateObjectResult {
    Ok,
    DuplicateId,
}

#[derive(Debug, Clone)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
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
pub struct ObjectCreatedEvent {
    pub id: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ObjectDestroyedEvent {
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub enum CreateServiceResult {
    Ok,
    DuplicateId,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug, Clone)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[derive(Debug, Clone)]
pub enum DestroyServiceResult {
    Ok,
    InvalidService,
    InvalidObject,
    ForeignObject,
}

#[derive(Debug, Clone)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[derive(Debug, Clone)]
pub struct ServiceCreatedEvent {
    pub object_id: Uuid,
    pub id: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ServiceDestroyedEvent {
    pub object_id: Uuid,
    pub id: Uuid,
}
