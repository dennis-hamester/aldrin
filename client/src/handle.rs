pub(crate) mod request;

use crate::bus_listener::BusListener;
use crate::channel::{
    PendingReceiver, PendingSender, ReceiverInner, SenderInner, UnclaimedReceiver, UnclaimedSender,
};
use crate::error::InvalidFunctionResult;
use crate::events::{EventsId, EventsRequest};
use crate::{Error, Events, Object, Service};
use aldrin_proto::message::{
    AddChannelCapacity, CallFunctionResult, DestroyObjectResult, QueryServiceVersionResult,
    SubscribeEventResult,
};
use aldrin_proto::{
    ChannelCookie, ChannelEnd, Deserialize, ObjectCookie, ObjectId, ObjectUuid, Serialize,
    SerializedValue, ServiceId, ServiceUuid,
};
use futures_channel::mpsc::UnboundedSender;
use futures_channel::oneshot;
use request::{
    CallFunctionReplyRequest, CallFunctionRequest, ClaimReceiverRequest, ClaimSenderRequest,
    CloseChannelEndRequest, CreateClaimedReceiverRequest, CreateObjectRequest,
    CreateServiceRequest, DestroyObjectRequest, DestroyServiceRequest, EmitEventRequest,
    HandleRequest, QueryServiceVersionRequest, SendItemRequest, SubscribeEventRequest,
    UnsubscribeEventRequest,
};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Handle to a client.
///
/// After connecting a [`Client`](crate::Client) to a broker, [`Handle`s](Handle) are used to
/// interact with it. The first [`Handle`] can be acquired with
/// [`Client::handle`](crate::Client::handle). After that, [`Handle`s](Handle) can be cloned
/// cheaply.
///
/// The [`Client`](crate::Client) will automatically shut down when the last [`Handle`] has been
/// dropped.
///
/// # Examples
///
/// ```
/// use aldrin_client::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let mut handle = broker.clone();
/// # let (async_transport, t2) = aldrin_channel::unbounded();
/// # let conn = tokio::spawn(async move { handle.connect(t2).await });
/// // Connect to the broker:
/// let client = Client::connect(async_transport).await?;
///
/// // Acquire the first handle:
/// let handle = client.handle().clone();
///
/// // Run the client, which consumes it and leaves only the handle for interacting with it:
/// tokio::spawn(client.run());
/// # tokio::spawn(conn.await??.run());
///
/// // Handles are cheap to clone:
/// let handle2 = handle.clone();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Handle {
    send: UnboundedSender<HandleRequest>,
}

impl Handle {
    pub(crate) fn new(send: UnboundedSender<HandleRequest>) -> Self {
        Handle { send }
    }

    /// Shuts down the client.
    ///
    /// Shutdown happens asynchronously, in the sense that when this function returns, the
    /// [`Client`](crate::Client) has only been requested to shut down and not yet necessarily done
    /// so. As soon as [`Client::run`](crate::Client::run) returns, it has fully shut down.
    ///
    /// If the [`Client`](crate::Client) has already shut down (due to any reason), this function
    /// will not treat that as an error. This is different than most other functions, which would
    /// return [`Error::ClientShutdown`] instead.
    pub fn shutdown(&self) {
        self.send.unbounded_send(HandleRequest::Shutdown).ok();
    }

    /// Creates a new object on the bus.
    ///
    /// The `uuid` must not yet exists on the bus, or else [`Error::DuplicateObject`] will be
    /// returned. Use [`ObjectUuid::new_v4`] to create a new random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::Error;
    /// use aldrin_proto::ObjectUuid;
    /// use uuid::uuid;
    ///
    /// const OBJECT2_UUID: ObjectUuid = ObjectUuid(uuid!("6173e119-8066-4776-989b-145a5f16ed4c"));
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create an object with a random UUID:
    /// let object1 = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Create an object with a fixed UUID:
    /// let object2 = handle.create_object(OBJECT2_UUID).await?;
    ///
    /// // Using the same UUID again will cause an error:
    /// assert_eq!(
    ///     handle.create_object(OBJECT2_UUID).await.unwrap_err(),
    ///     Error::DuplicateObject(OBJECT2_UUID)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_object(&self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateObject(CreateObjectRequest {
                uuid,
                reply: send,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) async fn destroy_object(&self, id: ObjectId) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie: id.cookie,
                reply: send,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        let reply = recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_object_now(&self, cookie: ObjectCookie) {
        let (reply, _) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie,
                reply,
            }))
            .ok();
    }

    pub(crate) async fn create_service(
        &self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
        version: u32,
    ) -> Result<Service, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateService(CreateServiceRequest {
                object_id,
                service_uuid,
                version,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) async fn destroy_service(&self, id: ServiceId) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) fn destroy_service_now(&self, id: ServiceId) {
        let (reply, _) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }))
            .ok();
    }

    /// Calls a function on a service.
    ///
    /// The function with id `function` will be called with the arguments `args` on the service
    /// identified by `service_id`.
    ///
    /// The returned value of type [`PendingFunctionResult`] is a future which will resolve to the
    /// result of the function call.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_proto::Value;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_proto::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_proto::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Call function 1 with "1 + 2 = ?" as the argument.
    /// let result = handle.call_function::<_, u32, String>(service_id, 1, "1 + 2 = ?")?;
    /// # svc.next_function_call().await.unwrap().reply.ok(&3u32)?;
    ///
    /// // Await the result. The `?` here checks for errors on the protocol level, such as a
    /// // intermediate shutdown, or whether the function call was aborted by the callee.
    /// let result = result.await?;
    ///
    /// // Now, result is of type `Result<u32, String>`, directly representing the result of the
    /// // function call.
    /// match result {
    ///     Ok(ok) => assert_eq!(ok, 3),
    ///     Err(err) => panic!("Function call failed: {}.", err),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_function<Args, T, E>(
        &self,
        service_id: ServiceId,
        function: u32,
        args: &Args,
    ) -> Result<PendingFunctionResult<T, E>, Error>
    where
        Args: Serialize + ?Sized,
        T: Deserialize,
        E: Deserialize,
    {
        let value = SerializedValue::serialize(args)?;
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CallFunction(CallFunctionRequest {
                service_cookie: service_id.cookie,
                function,
                value,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(PendingFunctionResult::new(recv, service_id, function))
    }

    /// Calls an infallible function on a service.
    ///
    /// Use this method if the called function is guaranteed to never fail. If this is not true, and
    /// the function fails, then [`Error::InvalidFunctionResult`] will be returned.
    ///
    /// The returned value of type [`PendingFunctionValue`] is a future which will resolve to the
    /// value of the function call.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_proto::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_proto::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Call function 1 with "1 + 2 = ?" as the argument.
    /// let result = handle.call_infallible_function::<_, u32>(service_id, 1, "1 + 2 = ?")?;
    /// # svc.next_function_call().await.unwrap().reply.ok(&3u32)?;
    ///
    /// assert_eq!(result.await?, 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_infallible_function<Args, T>(
        &self,
        service_id: ServiceId,
        function: u32,
        args: &Args,
    ) -> Result<PendingFunctionValue<T>, Error>
    where
        Args: Serialize + ?Sized,
        T: Deserialize,
    {
        let value = SerializedValue::serialize(args)?;
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CallFunction(CallFunctionRequest {
                service_cookie: service_id.cookie,
                function,
                value,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(PendingFunctionValue::new(recv, service_id, function))
    }

    pub(crate) fn function_call_reply(
        &self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::CallFunctionReply(CallFunctionReplyRequest {
                serial,
                result,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Creates an Events object used to subscribe to service events.
    ///
    /// See [`Events`] for more information and usage examples.
    pub fn events(&self) -> Events {
        Events::new(self.clone())
    }

    pub(crate) async fn subscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
        sender: UnboundedSender<EventsRequest>,
    ) -> Result<(), Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SubscribeEvent(SubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
                sender,
                reply: rep_send,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = rep_recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            SubscribeEventResult::Ok => Ok(()),
            SubscribeEventResult::InvalidService => Err(Error::InvalidService(service_id)),
        }
    }

    pub(crate) fn unsubscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::UnsubscribeEvent(UnsubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Emits an events to subscribed clients.
    ///
    /// The event with the id `event` of the service identified by `service_id` will be emitted with
    /// the arguments `args` to all subscribed clients.
    ///
    /// Use [`Handle::events`] to subscribe to events.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin_proto::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_proto::ServiceUuid::new_v4(), 0).await?;
    /// # let service_id = svc.id();
    /// // Emit event 1 with argument "Hello, world!":
    /// handle.emit_event(service_id, 1, "Hello, world!")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn emit_event<T>(&self, service_id: ServiceId, event: u32, value: &T) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
    {
        let value = SerializedValue::serialize(value)?;
        self.send
            .unbounded_send(HandleRequest::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                value,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Queries the version of a service.
    ///
    /// If `service_id` does not name a valid service, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// use aldrin_proto::{ObjectUuid, ServiceUuid};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    /// let service = object.create_service(ServiceUuid::new_v4(), 2).await?;
    ///
    /// let version = handle.query_service_version(service.id()).await?;
    /// assert_eq!(version, Some(2));
    ///
    /// service.destroy().await?;
    /// let version = handle.query_service_version(service.id()).await?;
    /// assert_eq!(version, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_service_version(&self, service_id: ServiceId) -> Result<Option<u32>, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryServiceVersion(
                QueryServiceVersionRequest {
                    cookie: service_id.cookie,
                    reply,
                },
            ))
            .map_err(|_| Error::ClientShutdown)?;

        match recv.await.map_err(|_| Error::ClientShutdown)? {
            QueryServiceVersionResult::Ok(version) => Ok(Some(version)),
            QueryServiceVersionResult::InvalidService => Ok(None),
        }
    }

    /// Creates a channel and automatically claims the sender.
    ///
    /// When creating a channel, one of the two end must be claimed immediately. This function
    /// claims the sender. Use
    /// [`create_channel_with_claimed_receiver`](Self::create_channel_with_claimed_receiver) to
    /// claim the receiver instead.
    ///
    /// # Examples
    ///
    /// This example assumes that there are 2 clients, represented here by `handle1` and `handle2`.
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle1 = broker.add_client().await;
    /// # let handle2 = broker.add_client().await;
    /// // Client 1 creates the channel. It then unbinds the receiver and makes it available to
    /// // client 2. This will typically happen by returning it from a function call.
    /// let (sender, receiver) = handle1.create_channel_with_claimed_sender().await?;
    /// let receiver = receiver.unbind();
    ///
    /// // Client 2 gets access to the receiver, and then binds and claims it.
    /// let mut receiver = receiver.claim(handle2.clone(), 16).await?;
    ///
    /// // Meanwhile, client 1 waits for the receiver to be claimed.
    /// let mut sender = sender.established().await?;
    ///
    /// // The channel is now fully established and client 1 can send items to client 2.
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// sender.send_item(&3).await?;
    ///
    /// // Client 1 will close (or drop) the channel when it has nothing to send anymore.
    /// sender.close().await?;
    ///
    /// // Client 2 receives all values in order. The Result in the return values can indicate
    /// // conversion errors when an item isn't a u32.
    /// assert_eq!(receiver.next_item().await, Ok(Some(1)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(2)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(3)));
    ///
    /// // Client 2 can observe that the sender has been closed by receiving None. It follows by
    /// // also closing (or dropping) the receiver.
    /// assert_eq!(receiver.next_item().await, Ok(None));
    /// receiver.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_channel_with_claimed_sender<T>(
        &self,
    ) -> Result<(PendingSender<T>, UnclaimedReceiver<T>), Error>
    where
        T: Serialize + Deserialize,
    {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateClaimedSender(reply))
            .map_err(|_| Error::ClientShutdown)?;

        let (sender, receiver) = recv.await.map_err(|_| Error::ClientShutdown)?;

        Ok((PendingSender::new(sender), UnclaimedReceiver::new(receiver)))
    }

    /// Creates a channel and automatically claims the receiver.
    ///
    /// When creating a channel, one of the two end must be claimed immediately. This function
    /// claims the receiver. Use
    /// [`create_channel_with_claimed_sender`](Self::create_channel_with_claimed_sender) to claim
    /// the sender instead.
    ///
    /// A `capacity` of 0 is treated as if 1 was specificed instead.
    ///
    /// # Examples
    ///
    /// See [`create_channel_with_claimed_sender`](Self::create_channel_with_claimed_sender) for an
    /// example.
    pub async fn create_channel_with_claimed_receiver<T>(
        &self,
        capacity: u32,
    ) -> Result<(UnclaimedSender<T>, PendingReceiver<T>), Error>
    where
        T: Serialize + Deserialize,
    {
        let capacity = NonZeroU32::new(capacity).unwrap_or(NonZeroU32::new(1).unwrap());

        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateClaimedReceiver(
                CreateClaimedReceiverRequest { capacity, reply },
            ))
            .map_err(|_| Error::ClientShutdown)?;

        let (sender, receiver) = recv.await.map_err(|_| Error::ClientShutdown)?;

        Ok((UnclaimedSender::new(sender), PendingReceiver::new(receiver)))
    }

    pub(crate) fn close_channel_end(
        &self,
        cookie: ChannelCookie,
        end: ChannelEnd,
        claimed: bool,
    ) -> Result<CloseChannelEndFuture, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CloseChannelEnd(CloseChannelEndRequest {
                cookie,
                end,
                claimed,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        Ok(CloseChannelEndFuture(recv))
    }

    pub(crate) async fn claim_sender(&self, cookie: ChannelCookie) -> Result<SenderInner, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::ClaimSender(ClaimSenderRequest {
                cookie,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) async fn claim_receiver(
        &self,
        cookie: ChannelCookie,
        capacity: u32,
    ) -> Result<ReceiverInner, Error> {
        let capacity = NonZeroU32::new(capacity).unwrap_or(NonZeroU32::new(1).unwrap());

        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::ClaimReceiver(ClaimReceiverRequest {
                cookie,
                capacity,
                reply,
            }))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)?
    }

    pub(crate) fn send_item(
        &self,
        cookie: ChannelCookie,
        value: SerializedValue,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::SendItem(SendItemRequest { cookie, value }))
            .map_err(|_| Error::ClientShutdown)
    }

    pub(crate) fn add_channel_capacity(
        &self,
        cookie: ChannelCookie,
        capacity: u32,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::AddChannelCapacity(AddChannelCapacity {
                cookie,
                capacity,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Synchronizes with the client.
    ///
    /// This function ensures that all previous requests to the client have been processed. There
    /// are some occasions in which requests are sent outside of an async context, e.g. when
    /// dropping values such as [`Object`]. By synchronizing with the client, it is possible to
    /// ensure that it has processed such a non-async request.
    ///
    /// See also [`sync_broker`](Self::sync_broker), which ensures that such requests have been
    /// processed by the broker.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_proto::ObjectUuid;
    /// use std::mem;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let obj = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Dropping obj will request the client to destroy the object.
    /// mem::drop(obj);
    ///
    /// // Ensure the request has actually been processed by the client.
    /// handle.sync_client().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync_client(&self) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SyncClient(reply))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)
    }

    /// Synchronizes with the broker.
    ///
    /// Certain requests such as emitting an event or sending an item on a channel don't synchronize
    /// with the broker in the same way as e.g. creating an object does. This function can be used
    /// to ensure that such a request has been processed by the broker.
    ///
    /// See also [`sync_client`](Self::sync_client), which ensures only that such requests have been
    /// processed by the client.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::{ObjectUuid, ServiceUuid};
    /// use std::mem;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(ObjectUuid::new_v4()).await?;
    /// # let service = obj.create_service(ServiceUuid::new_v4(), 0).await?;
    ///
    /// handle.emit_event(service.id(), 0, "Hi!")?;
    ///
    /// // Synchronize with the broker to ensure that the event has actually been processed.
    /// handle.sync_broker().await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync_broker(&self) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SyncBroker(reply))
            .map_err(|_| Error::ClientShutdown)?;

        recv.await.map_err(|_| Error::ClientShutdown)
    }

    /// TODO
    pub async fn create_bus_listener(&self) -> Result<BusListener, Error> {
        todo!()
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        self.send.unbounded_send(HandleRequest::HandleCloned).ok();
        Handle {
            send: self.send.clone(),
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.send.unbounded_send(HandleRequest::HandleDropped).ok();
    }
}

/// Future to await the result of a function call.
///
/// The future resolves to the type `Result<Result<T, E>, Error>`. The outer `Result<_, Error>`
/// represents the success or failure ([`Error`]) on the protocol and client library level. The
/// inner `Result<T, E>` represents the actual result of the function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingFunctionResult<T, E>
where
    T: Deserialize,
    E: Deserialize,
{
    recv: oneshot::Receiver<CallFunctionResult>,
    service_id: ServiceId,
    function: u32,
    _res: PhantomData<fn() -> (T, E)>,
}

impl<T, E> PendingFunctionResult<T, E>
where
    T: Deserialize,
    E: Deserialize,
{
    pub(crate) fn new(
        recv: oneshot::Receiver<CallFunctionResult>,
        service_id: ServiceId,
        function: u32,
    ) -> Self {
        PendingFunctionResult {
            recv,
            service_id,
            function,
            _res: PhantomData,
        }
    }
}

impl<T, E> Future for PendingFunctionResult<T, E>
where
    T: Deserialize,
    E: Deserialize,
{
    type Output = Result<Result<T, E>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let res = match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(_)) => return Poll::Ready(Err(Error::ClientShutdown)),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(match res {
            CallFunctionResult::Ok(t) => match t.deserialize() {
                Ok(t) => Ok(Ok(t)),
                Err(_) => Err(InvalidFunctionResult {
                    service_id: self.service_id,
                    function: self.function,
                }
                .into()),
            },
            CallFunctionResult::Err(e) => match e.deserialize() {
                Ok(e) => Ok(Err(e)),
                Err(_) => Err(InvalidFunctionResult {
                    service_id: self.service_id,
                    function: self.function,
                }
                .into()),
            },
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(self.service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(self.service_id, self.function))
            }
            CallFunctionResult::InvalidArgs => {
                Err(Error::InvalidArgs(self.service_id, self.function))
            }
        })
    }
}

impl<T, E> fmt::Debug for PendingFunctionResult<T, E>
where
    T: Deserialize,
    E: Deserialize,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingFunctionResult")
            .field("recv", &self.recv)
            .field("service_id", &self.service_id)
            .field("function", &self.function)
            .finish()
    }
}

/// Future to await the result of an infallible function call.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PendingFunctionValue<T>
where
    T: Deserialize,
{
    recv: oneshot::Receiver<CallFunctionResult>,
    service_id: ServiceId,
    function: u32,
    _res: PhantomData<fn() -> T>,
}

impl<T> PendingFunctionValue<T>
where
    T: Deserialize,
{
    pub(crate) fn new(
        recv: oneshot::Receiver<CallFunctionResult>,
        service_id: ServiceId,
        function: u32,
    ) -> Self {
        PendingFunctionValue {
            recv,
            service_id,
            function,
            _res: PhantomData,
        }
    }
}

impl<T> Future for PendingFunctionValue<T>
where
    T: Deserialize,
{
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let res = match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(_)) => return Poll::Ready(Err(Error::ClientShutdown)),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(match res {
            CallFunctionResult::Ok(t) => match t.deserialize() {
                Ok(t) => Ok(t),
                Err(_) => Err(InvalidFunctionResult {
                    service_id: self.service_id,
                    function: self.function,
                }
                .into()),
            },
            CallFunctionResult::Err(_) => Err(InvalidFunctionResult {
                service_id: self.service_id,
                function: self.function,
            }
            .into()),
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(self.service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(self.service_id, self.function))
            }
            CallFunctionResult::InvalidArgs => {
                Err(Error::InvalidArgs(self.service_id, self.function))
            }
        })
    }
}

impl<T> fmt::Debug for PendingFunctionValue<T>
where
    T: Deserialize,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingFunctionResult")
            .field("recv", &self.recv)
            .field("service_id", &self.service_id)
            .field("function", &self.function)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct CloseChannelEndFuture(oneshot::Receiver<Result<(), Error>>);

impl Future for CloseChannelEndFuture {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(Ok(Ok(()))) => Poll::Ready(Ok(())),
            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(Err(oneshot::Canceled)) => Poll::Ready(Err(Error::ClientShutdown)),
            Poll::Pending => Poll::Pending,
        }
    }
}
