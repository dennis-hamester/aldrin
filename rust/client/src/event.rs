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

use super::{FunctionCallReceiver, ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use aldrin_proto::*;
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(crate) enum Event {
    Shutdown,
    CreateObject(ObjectUuid, oneshot::Sender<CreateObjectResult>),
    DestroyObject(ObjectId, oneshot::Sender<DestroyObjectResult>),
    SubscribeObjectsCreated(mpsc::Sender<ObjectId>, bool),
    SubscribeObjectsDestroyed(mpsc::Sender<ObjectId>),
    CreateService(
        ObjectId,
        ServiceUuid,
        usize,
        oneshot::Sender<(CreateServiceResult, Option<FunctionCallReceiver>)>,
    ),
    DestroyService(ServiceId, oneshot::Sender<DestroyServiceResult>),
    SubscribeServicesCreated(mpsc::Sender<(ObjectId, ServiceId)>, bool),
    SubscribeServicesDestroyed(mpsc::Sender<(ObjectId, ServiceId)>),
    CallFunction(ServiceId, u32, Value, oneshot::Sender<CallFunctionResult>),
    FunctionCallReply(u32, CallFunctionResult),
}
