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
    id: ServiceId,
    inner: Option<Inner>,
}

impl Service {
    pub(crate) fn new(
        id: ServiceId,
        client: Handle,
        function_calls: Receiver<(u32, Value, u32)>,
    ) -> Self {
        Service {
            id,
            inner: Some(Inner {
                client,
                function_calls,
            }),
        }
    }

    pub fn id(&self) -> ServiceId {
        self.id
    }

    pub fn handle(&self) -> Option<&Handle> {
        self.inner.as_ref().map(|i| &i.client)
    }

    pub async fn destroy(&mut self) -> Result<(), Error> {
        let inner = self.inner.as_mut().ok_or(Error::InvalidService(self.id))?;
        let res = inner.client.destroy_service(self.id).await;
        if res.is_ok() {
            self.inner.take();
        }
        res
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            inner.client.destroy_service_now(self.id.cookie);
        }
    }
}

impl Stream for Service {
    type Item = FunctionCall;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<FunctionCall>> {
        let inner = match self.inner.as_mut() {
            Some(inner) => inner,
            None => return Poll::Ready(None),
        };

        let function_calls = Pin::new(&mut inner.function_calls);
        function_calls.poll_next(cx).map(|r| {
            r.map(|(function, args, serial)| {
                FunctionCall::new(function, args, inner.client.clone(), serial)
            })
        })
    }
}

#[derive(Debug)]
struct Inner {
    client: Handle,
    function_calls: Receiver<(u32, Value, u32)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ServiceId {
    pub object_id: ObjectId,
    pub uuid: ServiceUuid,
    pub cookie: ServiceCookie,
}

impl ServiceId {
    pub(crate) fn new(object_id: ObjectId, uuid: ServiceUuid, cookie: ServiceCookie) -> Self {
        ServiceId {
            object_id,
            uuid,
            cookie,
        }
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
#[non_exhaustive]
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
