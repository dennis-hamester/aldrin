#[cfg(test)]
mod test;

use crate::{Error, Handle};
use aldrin_proto::message::CallFunctionResult;
use aldrin_proto::{Serialize, SerializedValue, ServiceId};
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::{FusedStream, Stream};
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Owned service on the bus.
///
/// [`Service`s](Service) are associated with an [`Object`](crate::Object) and created with
/// [`Object::create_service`](crate::Object::create_service). [`Service`s](Service) can be
/// destroyed again by calling [`destroy`](Service::destroy), by dropping them, or implicitly when
/// the [`Object`](crate::Object) is destroyed.
///
/// [`Service`] exposes an asynchronous stream of incoming [`FunctionCall`s](FunctionCall) via its
/// implementation of the [`Stream`] trait.
///
/// Events can be emitted directly with [`Handle::emit_event`]. This is available on [`Handle`],
/// because usually [`Service`] is borrowed mutably to wait for function calls. The [`ServiceId`]
/// required for [`Handle::emit_event`] can be obtained with e.g. [`Service::id`].
///
/// # Examples
///
/// Creating and destroying [`Service`s](Service):
///
/// ```
/// use aldrin_client::Error;
/// use aldrin_proto::{ObjectUuid, ServiceUuid};
/// use std::mem;
/// use uuid::uuid;
///
/// const SERVICE_UUID: ServiceUuid = ServiceUuid(uuid!("f88f1706-9609-42a4-8796-4e7bb8c3ef24"));
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// let object = handle.create_object(ObjectUuid::new_v4()).await?;
///
/// // Create a service and destroy it again explicitly:
/// let service = object.create_service(SERVICE_UUID, 1).await?;
/// service.destroy().await?;
///
/// // Destroy a service implicitly by dropping it:
/// let service = object.create_service(SERVICE_UUID, 1).await?;
/// mem::drop(service);
///
/// // Destroy a service implicitly by dropping the object:
/// let service = object.create_service(SERVICE_UUID, 1).await?;
/// let service_id = service.id();
/// mem::drop(object);
/// assert_eq!(service.destroy().await, Err(Error::InvalidService(service_id)));
/// # Ok(())
/// # }
/// ```
///
/// The following is a small chat server example, which shows how to handle function call on a
/// service and how to emit events.
///
/// ```
/// use aldrin_proto::{ObjectUuid, ServiceUuid};
/// use std::collections::HashSet;
/// use uuid::uuid;
///
/// const CHAT_UUID: ServiceUuid = ServiceUuid(uuid!("91334d42-7045-4292-99dc-9fd89c5f104f"));
///
/// // Functions
/// const SHUTDOWN: u32 = 1;
/// const JOIN_CHAT: u32 = 2;
/// const LEAVE_CHAT: u32 = 3;
/// const LIST_USERS: u32 = 4;
/// const SEND_MESSAGE: u32 = 5;
///
/// // Events
/// const JOINED_CHAT: u32 = 1;
/// const LEFT_CHAT: u32 = 2;
/// const MESSAGE_SENT: u32 = 3;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// // Create object and our chat service:
/// let object = handle.create_object(ObjectUuid::new_v4()).await?;
/// let mut service = object.create_service(CHAT_UUID, 1).await?;
/// let service_id = service.id();
///
/// // HashSet to keep track of users:
/// let mut users = HashSet::new();
///
/// # handle.call_function::<_, (), ()>(service.id(), SHUTDOWN, &())?;
/// // Iterate (asynchronously) over incoming function calls. `func` is of type `FunctionCall`,
/// // which contains the function's id, the arguments, and a reply object.
/// while let Some(func) = service.next_function_call().await {
///     match func.id {
///         SHUTDOWN => break,
///
///         JOIN_CHAT => {
///             let name: String = func.args.deserialize()?;
///             if users.insert(name.clone()) {
///                 // Emit an event that a new user with the given name joined:
///                 handle.emit_event(service_id, JOINED_CHAT, &name)?;
///
///                 func.reply.ok(&())?;
///             } else {
///                 // Signal that the name is already taken.
///                 func.reply.err(&())?;
///             }
///         }
///
///         LEAVE_CHAT => {
///             let name: String = func.args.deserialize()?;
///             if users.remove(&name) {
///                 // Emit an event that a user with the given name left:
///                 handle.emit_event(service_id, LEFT_CHAT, &name)?;
///
///                 func.reply.ok(&())?;
///             } else {
///                 // Signal that the name is not known.
///                 func.reply.err(&())?;
///             }
///         }
///
///         LIST_USERS => func.reply.ok(&users)?,
///
///         SEND_MESSAGE => {
///             // Broadcast the message:
///             let message = func.args.deserialize()?;
///             handle.emit_event(service_id, MESSAGE_SENT, &message)?;
///             func.reply.ok(&())?;
///         }
///
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Service {
    id: ServiceId,
    version: u32,
    client: Handle,
    function_calls: UnboundedReceiver<RawFunctionCall>,
}

impl Service {
    pub(crate) fn new(
        id: ServiceId,
        version: u32,
        client: Handle,
        function_calls: UnboundedReceiver<RawFunctionCall>,
    ) -> Self {
        Service {
            id,
            version,
            client,
            function_calls,
        }
    }

    /// Returns the id of the service.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the version of the service.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns a handle to the client that was used to create the service.
    pub fn handle(&self) -> &Handle {
        &self.client
    }

    /// Destroys the service.
    ///
    /// If the [`Service`] has already been destroyed, then [`Error::InvalidService`] is returned.
    pub async fn destroy(&self) -> Result<(), Error> {
        self.client.destroy_service(self.id).await
    }

    /// Polls for the next function call.
    pub fn poll_next_function_call(&mut self, cx: &mut Context) -> Poll<Option<FunctionCall>> {
        Pin::new(&mut self.function_calls).poll_next(cx).map(|r| {
            r.map(|req| FunctionCall::new(req.function, req.value, self.client.clone(), req.serial))
        })
    }

    /// Returns the next function call.
    pub async fn next_function_call(&mut self) -> Option<FunctionCall> {
        future::poll_fn(|cx| self.poll_next_function_call(cx)).await
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.client.destroy_service_now(self.id);
    }
}

impl Stream for Service {
    type Item = FunctionCall;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<FunctionCall>> {
        self.poll_next_function_call(cx)
    }
}

impl FusedStream for Service {
    fn is_terminated(&self) -> bool {
        self.function_calls.is_terminated()
    }
}

/// Function call received by a service.
///
/// [`FunctionCall`s](FunctionCall) can be received with the [`Stream`] implementation of
/// [`Service`].
///
/// See [`Service`] for usage examples.
#[derive(Debug)]
pub struct FunctionCall {
    /// Id of the called function.
    pub id: u32,

    /// Arguments passed to called function.
    pub args: SerializedValue,

    /// Reply object, used to set the return value of the function call.
    pub reply: FunctionCallReply,
}

impl FunctionCall {
    pub(crate) fn new(id: u32, args: SerializedValue, client: Handle, serial: u32) -> Self {
        FunctionCall {
            id,
            args,
            reply: FunctionCallReply::new(client, serial),
        }
    }
}

/// Helper used to reply to a pending service function call.
///
/// Every [`FunctionCall`] contains a [`FunctionCallReply`]. It can be used once to set the return
/// value of the function call.
///
/// When [`FunctionCallReply`] is dropped (as opposed to consumed by one of its methods),
/// [`abort`](FunctionCallReply::abort) will be called implicitly.
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

    /// Sets the function call's reply.
    pub fn set<T, E>(self, res: Result<&T, &E>) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
        E: Serialize + ?Sized,
    {
        match res {
            Ok(value) => self.ok(value),
            Err(value) => self.err(value),
        }
    }

    /// Signals that the function call was successful.
    pub fn ok<T: Serialize + ?Sized>(mut self, value: &T) -> Result<(), Error> {
        let res = CallFunctionResult::ok_with_serialize_value(value)?;
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the function call has failed.
    pub fn err<T: Serialize + ?Sized>(mut self, value: &T) -> Result<(), Error> {
        let res = CallFunctionResult::err_with_serialize_value(value)?;
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Aborts the function call.
    ///
    /// The caller will be still be notified that the call was aborted.
    pub fn abort(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Aborted)
    }

    /// Signals that an invalid function has been called.
    ///
    /// The function, as identified by [`FunctionCall::id`], may be invalid or unexpected by the
    /// service.
    pub fn invalid_function(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidFunction)
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidArgs)
    }
}

impl Drop for FunctionCallReply {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            client
                .function_call_reply(self.serial, CallFunctionResult::Aborted)
                .ok();
        }
    }
}

#[derive(Debug)]
pub(crate) struct RawFunctionCall {
    pub serial: u32,
    pub function: u32,
    pub value: SerializedValue,
}
