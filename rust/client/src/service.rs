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

use super::{Error, Handle, ObjectId};
use aldrin_proto::{CallFunctionResult, Value};
use futures_channel::mpsc::Receiver;
use futures_core::stream::Stream;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

#[derive(Debug)]
pub struct Service {
    object_id: ObjectId,
    id: ServiceId,
    client: Handle,
    destroyed: bool,
    function_calls: Receiver<(u32, Value, u32)>,
}

impl Service {
    pub(crate) fn new(
        object_id: ObjectId,
        id: ServiceId,
        client: Handle,
        function_calls: Receiver<(u32, Value, u32)>,
    ) -> Self {
        Service {
            object_id,
            id,
            client,
            destroyed: false,
            function_calls,
        }
    }

    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }

    pub fn id(&self) -> ServiceId {
        self.id
    }

    pub async fn destroy(&mut self) -> Result<(), Error> {
        let res = self.client.destroy_service(self.object_id, self.id).await;
        self.destroyed = true;
        res
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if !self.destroyed {
            self.client.destroy_service_now(self.id);
            self.destroyed = true;
        }
    }
}

impl Stream for Service {
    type Item = FunctionCall;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<FunctionCall>> {
        let res = Pin::new(&mut self.function_calls).poll_next(cx);
        res.map(|r| {
            r.map(|(function, args, serial)| {
                FunctionCall::new(function, args, self.client.clone(), serial)
            })
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceId {
    pub uuid: ServiceUuid,
    pub cookie: ServiceCookie,
}

impl ServiceId {
    pub fn new(uuid: ServiceUuid, cookie: ServiceCookie) -> Self {
        ServiceId { uuid, cookie }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceUuid(pub Uuid);

impl fmt::Display for ServiceUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceCookie(pub Uuid);

impl fmt::Display for ServiceCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub struct FunctionCall {
    pub id: u32,
    pub args: Value,
    pub reply: FunctionCallReply,
}

impl FunctionCall {
    pub(crate) fn new(id: u32, args: Value, client: Handle, serial: u32) -> Self {
        FunctionCall {
            id,
            args,
            reply: FunctionCallReply::new(client, serial),
        }
    }
}

#[derive(Debug)]
pub struct FunctionCallReply {
    client: Option<Handle>,
    serial: u32,
}

impl FunctionCallReply {
    pub(crate) fn new(client: Handle, serial: u32) -> Self {
        FunctionCallReply {
            client: Some(client),
            serial,
        }
    }

    pub async fn ok(mut self, res: Value) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Ok(res))
            .await
    }

    pub async fn err(mut self, res: Value) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Err(res))
            .await
    }

    pub async fn abort(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Aborted)
            .await
    }

    pub async fn invalid_function(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidFunction)
            .await
    }

    pub async fn invalid_args(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidArgs)
            .await
    }
}

impl Drop for FunctionCallReply {
    fn drop(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.abort_function_call_now(self.serial);
        }
    }
}
