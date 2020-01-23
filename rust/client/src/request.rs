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

use super::{
    EventsId, EventsRequest, FunctionCallReceiver, ObjectCookie, ObjectId, ObjectUuid,
    ServiceCookie, ServiceId, ServiceUuid, SubscribeMode,
};
use aldrin_proto::*;
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(crate) enum Request {
    Shutdown,
    CreateObject(ObjectUuid, oneshot::Sender<CreateObjectResult>),
    DestroyObject(ObjectCookie, oneshot::Sender<DestroyObjectResult>),
    SubscribeObjectsCreated(mpsc::Sender<ObjectId>, SubscribeMode),
    SubscribeObjectsDestroyed(mpsc::Sender<ObjectId>),
    CreateService(
        ObjectCookie,
        ServiceUuid,
        usize,
        oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ),
    DestroyService(ServiceCookie, oneshot::Sender<DestroyServiceResult>),
    SubscribeServicesCreated(mpsc::Sender<ServiceId>, SubscribeMode),
    SubscribeServicesDestroyed(mpsc::Sender<ServiceId>),
    CallFunction(
        ServiceCookie,
        u32,
        Value,
        oneshot::Sender<CallFunctionResult>,
    ),
    FunctionCallReply(u32, CallFunctionResult),
    SubscribeEvent(SubscribeEventRequest),
    UnsubscribeEvent(UnsubscribeEventRequest),
    EmitEvent(EmitEventRequest),
}

#[derive(Debug)]
pub(crate) struct SubscribeEventRequest {
    pub events_id: EventsId,
    pub service_cookie: ServiceCookie,
    pub id: u32,
    pub sender: mpsc::Sender<EventsRequest>,
    pub reply: oneshot::Sender<SubscribeEventResult>,
}

#[derive(Debug)]
pub(crate) struct UnsubscribeEventRequest {
    pub events_id: EventsId,
    pub service_cookie: ServiceCookie,
    pub id: u32,
}

#[derive(Debug)]
pub(crate) struct EmitEventRequest {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub args: Value,
}
