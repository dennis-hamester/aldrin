use super::{Error, Handle, ObjectId};
use aldrin_proto::{CallFunctionResult, Value};
use futures_channel::mpsc::Receiver;
use futures_core::stream::Stream;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

/// Owned service.
///
/// Services are always associated with an [`Object`](super::Object) and created by
/// [`Object::create_service`](super::Object::create_service). Services are destroyed again by
/// calling [`destroy`](Service::destroy), by dropping them, or implicitly when the
/// [`Object`](super::Object) is destroyed.
///
/// To handle incoming function calls, iterate asynchronously over a service with the [`Stream`]
/// trait, which then yields [`FunctionCall`s](FunctionCall).
///
/// Events can be emitted directly with [`Handle::emit_event`]. This is available on [`Handle`],
/// because usually [`Service`] is borrowed mutably to wait for function calls. The [`ServiceId`]
/// required for [`Handle::emit_event`] can be obtained with e.g. [`Service::id`].
///
/// # Examples
///
/// ```no_run
/// // For StreamExt::next()
/// use futures::stream::StreamExt;
///
/// // Create a new service with a random UUID.
/// let mut service = object.create_service(ServiceUuid(Uuid::new_v4()), FIFO_SIZE).await?;
///
/// // Handle next function call.
/// if let Some(call) = service.next().await {
///     match call.id {
///         1 => {
///             // ...
///             handle.emit_event(service.id(), 1, Value::None).await?;
///             call.reply.ok(res).await?
///         }
///
///         _ => call.reply.invalid_function().await?,
///     }
/// }
///
/// // Destroy the service again.
/// service.destroy().await?;
/// ```
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

    /// Returns the id of the service.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns a handle to the client that was used to create the service.
    ///
    /// `None` is returned after the [`Service`] has been manually [destroyed](Service::destroy).
    pub fn handle(&self) -> Option<&Handle> {
        self.inner.as_ref().map(|i| &i.client)
    }

    /// Destroys the service.
    ///
    /// If the service has already been destroyed, [`Error::InvalidService`] is returned.
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

/// Id of a service.
///
/// A [`ServiceId`] consists of an arbitrary UUID, and a cookie chosen by the server. The
/// combination of both is unique at all times across the whole Aldrin bus.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ServiceId {
    /// Object id to which the service belongs.
    pub object_id: ObjectId,

    /// UUID of the service.
    pub uuid: ServiceUuid,

    /// Cookie of the service.
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

/// UUID of a service.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceUuid(pub Uuid);

impl fmt::Display for ServiceUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Cookie of a service.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ServiceCookie(pub Uuid);

impl fmt::Display for ServiceCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Function call received by a service.
///
/// Function calls can be received with the [`Stream`] implementation of [`Service`].
#[derive(Debug)]
pub struct FunctionCall {
    /// Id of the called function.
    pub id: u32,

    /// Arguments passed to called function.
    pub args: Value,

    /// Used to set the reply of a function call.
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

/// Helper used to reply to a pending service function call.
///
/// Every [`FunctionCall`] caries a reply object as well. It can be used once to set the reply of
/// the function call.
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

    /// Signal that the function call was successful.
    pub async fn ok(mut self, res: Value) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Ok(res))
            .await
    }

    /// Signal that the function call failed.
    pub async fn err(mut self, res: Value) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Err(res))
            .await
    }

    /// Abort the function call.
    ///
    /// The caller will be still be notified that the call was aborted.
    pub async fn abort(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Aborted)
            .await
    }

    /// Signal that an invalid function has been called.
    ///
    /// The function itself is identified by [`FunctionCall::id`], which might be invalid or
    /// unexpected by the service.
    pub async fn invalid_function(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidFunction)
            .await
    }

    /// Signal that a invalid arguments were passed to the function.
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
