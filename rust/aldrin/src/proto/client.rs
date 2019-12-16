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
pub enum ClientMessage {
    Connect(Connect),
    CreateObject(CreateObject),
    DestroyObject(DestroyObject),
    SubscribeObjectsCreated(SubscribeObjectsCreated),
    UnsubscribeObjectsCreated,
    SubscribeObjectsDestroyed,
    UnsubscribeObjectsDestroyed,
    CreateService(CreateService),
    DestroyService(DestroyService),
    SubscribeServicesCreated(SubscribeServicesCreated),
    UnsubscribeServicesCreated,
    SubscribeServicesDestroyed,
    UnsubscribeServicesDestroyed,
}

#[derive(Debug, Clone)]
pub struct Connect {
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct CreateObject {
    pub serial: u32,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct DestroyObject {
    pub serial: u32,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SubscribeObjectsCreated {
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CreateService {
    pub serial: u32,
    pub object_id: Uuid,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct DestroyService {
    pub serial: u32,
    pub object_id: Uuid,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SubscribeServicesCreated {
    pub serial: Option<u32>,
}
