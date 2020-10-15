#[cfg(test)]
mod test;

use crate::{Error, Handle, ObjectCookie, ObjectId, ObjectUuid};
use aldrin_proto::{CallFunctionResult, ConversionError, FromValue, IntoValue, Value};
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::{FusedStream, Stream};
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

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
/// use aldrin_client::{Error, ObjectUuid, ServiceUuid};
/// use std::mem;
///
/// // f88f1706-9609-42a4-8796-4e7bb8c3ef24
/// const SERVICE_UUID: ServiceUuid = ServiceUuid::from_u128(0xf88f1706960942a487964e7bb8c3ef24);
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
/// use aldrin_client::{ObjectUuid, ServiceUuid};
/// use futures::stream::StreamExt;
/// use std::collections::HashSet;
///
/// // 91334d42-7045-4292-99dc-9fd89c5f104f
/// const CHAT_UUID: ServiceUuid = ServiceUuid::from_u128(0x91334d427045429299dc9fd89c5f104f);
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
/// # handle.call_function(service.id(), SHUTDOWN, ())?;
/// // Iterate (asynchronously) over incoming function calls. `func` is of type `FunctionCall`,
/// // which contains the function's id, the arguments, and a reply object.
/// while let Some(func) = service.next().await {
///     match func.id {
///         SHUTDOWN => break,
///
///         JOIN_CHAT => {
///             let name: String = func.args.convert()?;
///             if users.insert(name.clone()) {
///                 // Emit an event that a new user with the given name joined:
///                 handle.emit_event(service_id, JOINED_CHAT, name)?;
///
///                 func.reply.ok(())?;
///             } else {
///                 // Signal that the name is already taken.
///                 func.reply.err(())?;
///             }
///         }
///
///         LEAVE_CHAT => {
///             let name: String = func.args.convert()?;
///             if users.remove(&name) {
///                 // Emit an event that a user with the given name left:
///                 handle.emit_event(service_id, LEFT_CHAT, name)?;
///
///                 func.reply.ok(())?;
///             } else {
///                 // Signal that the name is not known.
///                 func.reply.err(())?;
///             }
///         }
///
///         LIST_USERS => func.reply.ok(users.clone())?,
///
///         SEND_MESSAGE => {
///             // Broadcast the message:
///             let message = func.args.convert()?;
///             handle.emit_event(service_id, MESSAGE_SENT, message)?;
///             func.reply.ok(())?;
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
    function_calls: UnboundedReceiver<(u32, Value, u32)>,
}

impl Service {
    pub(crate) fn new(
        id: ServiceId,
        version: u32,
        client: Handle,
        function_calls: UnboundedReceiver<(u32, Value, u32)>,
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
}

impl Drop for Service {
    fn drop(&mut self) {
        self.client.destroy_service_now(self.id);
    }
}

impl Stream for Service {
    type Item = FunctionCall;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<FunctionCall>> {
        Pin::new(&mut self.function_calls).poll_next(cx).map(|r| {
            r.map(|(function, args, serial)| {
                FunctionCall::new(function, args, self.client.clone(), serial)
            })
        })
    }
}

impl FusedStream for Service {
    fn is_terminated(&self) -> bool {
        self.function_calls.is_terminated()
    }
}

/// Id of a service.
///
/// An [`ServiceId`] consists of three parts:
/// - An [`ObjectId`], identifying the associated [`Object`](crate::Object) on the bus
/// - A [`ServiceUuid`], identifying the [`Service`] on the [`Object`](crate::Object)
/// - A [`ServiceCookie`], a random UUID chosen by the broker
///
/// It is important to point out, that when a service is destroyed and later created again with the
/// same [`ServiceUuid`], then the [`ServiceCookie`] and consequently the [`ServiceId`] will be
/// different. See [`ServiceCookie`] for more information.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceId {
    /// Id of the associated object.
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

impl From<aldrin_proto::ServiceId> for ServiceId {
    fn from(id: aldrin_proto::ServiceId) -> Self {
        ServiceId {
            object_id: ObjectId {
                uuid: ObjectUuid(id.object_uuid),
                cookie: ObjectCookie(id.object_cookie),
            },
            uuid: ServiceUuid(id.service_uuid),
            cookie: ServiceCookie(id.service_cookie),
        }
    }
}

impl From<ServiceId> for aldrin_proto::ServiceId {
    fn from(id: ServiceId) -> Self {
        aldrin_proto::ServiceId {
            object_uuid: id.object_id.uuid.0,
            object_cookie: id.object_id.cookie.0,
            service_uuid: id.uuid.0,
            service_cookie: id.cookie.0,
        }
    }
}

impl FromValue for ServiceId {
    fn from_value(v: Value) -> Result<ServiceId, ConversionError> {
        match v {
            Value::ServiceId(v) => Ok(v.into()),
            _ => Err(ConversionError),
        }
    }
}

impl IntoValue for ServiceId {
    fn into_value(self) -> Value {
        Value::ServiceId(self.into())
    }
}

/// UUID of a service.
///
/// [`ServiceUuid`s](ServiceUuid) are chosen by the user when
/// [creating](crate::Object::create_service) a service and must be unique among all services of an
/// [`Object`](crate::Object).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceUuid(pub Uuid);

impl ServiceUuid {
    /// Creates a ServiceUuid with a random v4 UUID.
    ///
    /// In general, random [`ServiceUuid`s](ServiceUuid) have limited use. But it may occasionally
    /// be convenient to create random [`ServiceUuid`s](ServiceUuid) in e.g. unit-tests.
    pub fn new_v4() -> Self {
        ServiceUuid(Uuid::new_v4())
    }

    /// Creates an ServiceUuid from an unsigned 128bit value in big-endian order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_client::ServiceUuid;
    /// // b94fb06d-791b-4aa9-afd1-9e2f69345eee
    /// let service_uuid = ServiceUuid::from_u128(0xb94fb06d791b4aa9afd19e2f69345eee);
    /// ```
    pub const fn from_u128(uuid: u128) -> Self {
        ServiceUuid(Uuid::from_u128(uuid))
    }
}

impl fmt::Display for ServiceUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Cookie of a service.
///
/// [`ServiceCookie`s](ServiceCookie) are chosen by the broker when
/// [creating](crate::Object::create_service) a [`Service`]. They help distinguish the [`Service`]
/// across time.
///
/// ```
/// use aldrin_client::ServiceUuid;
///
/// // 10e70a49-18ce-447c-949e-22fbd536a475
/// const SERVICE_UUID: ServiceUuid = ServiceUuid::from_u128(0x10e70a4918ce447c949e22fbd536a475);
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// # let mut object = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
/// // Create a service:
/// let service = object.create_service(SERVICE_UUID, 1).await?;
/// let service_id1 = service.id();
/// service.destroy().await?;
///
/// // Create the same service again:
/// let service = object.create_service(SERVICE_UUID, 1).await?;
/// let service_id2 = service.id();
/// service.destroy().await?;
///
/// // The service UUIDs will be equal:
/// assert_eq!(service_id1.uuid, SERVICE_UUID);
/// assert_eq!(service_id2.uuid, SERVICE_UUID);
///
/// // But the cookies will be different:
/// assert_ne!(service_id1.cookie, service_id2.cookie);
///
/// // Consequently, the ids will be different as well:
/// assert_ne!(service_id1, service_id2);
/// # Ok(())
/// # }
/// ```
///
/// In general, [`ServiceCookie`s](ServiceCookie) should be considered an implementation detail of
/// the Aldrin protocol and there is rarely a reason to deal with them manually.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceCookie(pub Uuid);

impl fmt::Display for ServiceCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
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
    pub args: Value,

    /// Reply object, used to set the return value of the function call.
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

    /// Signals that the function call was successful.
    pub fn ok(mut self, res: impl IntoValue) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Ok(res.into_value()))
    }

    /// Signals that the function call has failed.
    pub fn err(mut self, res: impl IntoValue) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Err(res.into_value()))
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
            client.abort_function_call_now(self.serial);
        }
    }
}
