pub(crate) mod request;

use crate::bus_listener::BusListener;
use crate::channel::{
    PendingReceiver, PendingSender, ReceiverInner, SenderInner, UnclaimedReceiver, UnclaimedSender,
};
use crate::core::message::{
    AddBusListenerFilter, AddChannelCapacity, CallFunctionResult, ClearBusListenerFilters,
    DestroyBusListenerResult, DestroyObjectResult, QueryServiceVersionResult,
    RemoveBusListenerFilter, StartBusListenerResult, StopBusListenerResult, SubscribeEventResult,
};
use crate::core::{
    BusListenerCookie, BusListenerFilter, BusListenerScope, ChannelCookie, ChannelEnd, Deserialize,
    ObjectCookie, ObjectId, ObjectUuid, ProtocolVersion, Serialize, SerializedValue, ServiceId,
    ServiceUuid,
};
use crate::discoverer::{Discoverer, DiscovererBuilder};
use crate::error::Error;
use crate::lifetime::{Lifetime, LifetimeId, LifetimeListener, LifetimeScope};
use crate::low_level::{EventListener, EventListenerId, EventListenerRequest, Proxy};
use crate::object::Object;
use crate::reply::Reply;
use crate::service::Service;
use futures_channel::mpsc::UnboundedSender;
use futures_channel::oneshot;
use request::{
    CallFunctionReplyRequest, CallFunctionRequest, ClaimReceiverRequest, ClaimSenderRequest,
    CloseChannelEndRequest, CreateClaimedReceiverRequest, CreateObjectRequest,
    CreateServiceRequest, DestroyBusListenerRequest, DestroyObjectRequest, DestroyServiceRequest,
    EmitEventRequest, HandleRequest, QueryServiceVersionRequest, SendItemRequest,
    StartBusListenerRequest, StopBusListenerRequest, SubscribeEventRequest,
    UnsubscribeEventRequest,
};
use std::future::Future;
use std::mem::MaybeUninit;
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
/// use aldrin::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio::TestBroker::new();
/// # let mut handle = broker.clone();
/// # let (async_transport, t2) = aldrin::core::channel::unbounded();
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
    /// return [`Error::Shutdown`] instead.
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
    /// use aldrin::Error;
    /// use aldrin::core::ObjectUuid;
    /// use uuid::uuid;
    ///
    /// const OBJECT2_UUID: ObjectUuid = ObjectUuid(uuid!("6173e119-8066-4776-989b-145a5f16ed4c"));
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
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
    ///     Error::DuplicateObject,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_object(&self, uuid: impl Into<ObjectUuid>) -> Result<Object, Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateObject(CreateObjectRequest {
                uuid: uuid.into(),
                reply: send,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) async fn destroy_object(&self, id: ObjectId) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie: id.cookie,
                reply: send,
            }))
            .map_err(|_| Error::Shutdown)?;

        let reply = recv.await.map_err(|_| Error::Shutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject),
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
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) async fn destroy_service(&self, id: ServiceId) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
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

    pub(crate) fn call<Args, T, E>(&self, id: ServiceId, function: u32, args: &Args) -> Reply<T, E>
    where
        Args: Serialize + ?Sized,
    {
        let (send, recv) = oneshot::channel();

        match SerializedValue::serialize(args) {
            Ok(value) => {
                let req = HandleRequest::CallFunction(CallFunctionRequest {
                    service_cookie: id.cookie,
                    function,
                    value,
                    reply: send,
                });

                let _ = self.send.unbounded_send(req);
            }

            Err(e) => {
                let _ = send.send(Err(e.into()));
            }
        }

        Reply::new(recv, function)
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
            .map_err(|_| Error::Shutdown)
    }

    /// Creates a new [`EventListener`].
    pub fn create_event_listener(&self) -> EventListener {
        EventListener::new(self.clone())
    }

    pub(crate) async fn subscribe_event(
        &self,
        listener_id: EventListenerId,
        service_id: ServiceId,
        id: u32,
        sender: UnboundedSender<EventListenerRequest>,
    ) -> Result<(), Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SubscribeEvent(SubscribeEventRequest {
                listener_id,
                service_cookie: service_id.cookie,
                id,
                sender,
                reply: rep_send,
            }))
            .map_err(|_| Error::Shutdown)?;
        let reply = rep_recv.await.map_err(|_| Error::Shutdown)?;
        match reply {
            SubscribeEventResult::Ok => Ok(()),
            SubscribeEventResult::InvalidService => Err(Error::InvalidService),
        }
    }

    pub(crate) fn unsubscribe_event(
        &self,
        listener_id: EventListenerId,
        service_id: ServiceId,
        id: u32,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::UnsubscribeEvent(UnsubscribeEventRequest {
                listener_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .map_err(|_| Error::Shutdown)
    }

    /// Emits an events to subscribed clients.
    ///
    /// The event with the id `event` of the service identified by `service_id` will be emitted with
    /// the arguments `args` to all subscribed clients.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(aldrin::core::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin::core::ServiceUuid::new_v4(), 0).await?;
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
            .map_err(|_| Error::Shutdown)
    }

    /// Queries the version of a service.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// use aldrin::core::{ObjectUuid, ServiceUuid};
    /// use aldrin::Error;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let object = handle.create_object(ObjectUuid::new_v4()).await?;
    /// let service = object.create_service(ServiceUuid::new_v4(), 2).await?;
    ///
    /// let version = handle.query_service_version(service.id()).await;
    /// assert_eq!(version, Ok(2));
    ///
    /// service.destroy().await?;
    /// let version = handle.query_service_version(service.id()).await;
    /// assert_eq!(version, Err(Error::InvalidService));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_service_version(&self, service_id: ServiceId) -> Result<u32, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryServiceVersion(
                QueryServiceVersionRequest {
                    cookie: service_id.cookie,
                    reply,
                },
            ))
            .map_err(|_| Error::Shutdown)?;

        match recv.await.map_err(|_| Error::Shutdown)? {
            QueryServiceVersionResult::Ok(version) => Ok(version),
            QueryServiceVersionResult::InvalidService => Err(Error::InvalidService),
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
    /// # use aldrin_test::tokio::TestBroker;
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
            .map_err(|_| Error::Shutdown)?;

        let (sender, receiver) = recv.await.map_err(|_| Error::Shutdown)?;

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
            .map_err(|_| Error::Shutdown)?;

        let (sender, receiver) = recv.await.map_err(|_| Error::Shutdown)?;

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
            .map_err(|_| Error::Shutdown)?;

        Ok(CloseChannelEndFuture(recv))
    }

    pub(crate) async fn claim_sender(&self, cookie: ChannelCookie) -> Result<SenderInner, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::ClaimSender(ClaimSenderRequest {
                cookie,
                reply,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
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
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) fn send_item(
        &self,
        cookie: ChannelCookie,
        value: SerializedValue,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::SendItem(SendItemRequest { cookie, value }))
            .map_err(|_| Error::Shutdown)
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
            .map_err(|_| Error::Shutdown)
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
    /// use aldrin::core::ObjectUuid;
    /// use std::mem;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
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
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
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
    /// # use aldrin::core::{ObjectUuid, ServiceUuid};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
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
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    /// Creates a new bus listener.
    ///
    /// Bus listeners enable monitoring the bus for events about the creation and destruction of
    /// objects and services. See [`BusListener`] for more information and usage examples.
    pub async fn create_bus_listener(&self) -> Result<BusListener, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateBusListener(reply))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    pub(crate) async fn destroy_bus_listener(
        &self,
        cookie: BusListenerCookie,
    ) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyBusListener(
                DestroyBusListenerRequest {
                    cookie,
                    reply: send,
                },
            ))
            .map_err(|_| Error::Shutdown)?;

        let reply = recv.await.map_err(|_| Error::Shutdown)?;
        match reply {
            DestroyBusListenerResult::Ok => Ok(()),
            DestroyBusListenerResult::InvalidBusListener => Err(Error::InvalidBusListener),
        }
    }

    pub(crate) fn destroy_bus_listener_now(&self, cookie: BusListenerCookie) {
        let (reply, _) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::DestroyBusListener(
                DestroyBusListenerRequest { cookie, reply },
            ))
            .ok();
    }

    pub(crate) fn add_bus_listener_filter(
        &self,
        cookie: BusListenerCookie,
        filter: BusListenerFilter,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::AddBusListenerFilter(AddBusListenerFilter {
                cookie,
                filter,
            }))
            .map_err(|_| Error::Shutdown)
    }

    pub(crate) fn remove_bus_listener_filter(
        &self,
        cookie: BusListenerCookie,
        filter: BusListenerFilter,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::RemoveBusListenerFilter(
                RemoveBusListenerFilter { cookie, filter },
            ))
            .map_err(|_| Error::Shutdown)
    }

    pub(crate) fn clear_bus_listener_filters(
        &self,
        cookie: BusListenerCookie,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::ClearBusListenerFilters(
                ClearBusListenerFilters { cookie },
            ))
            .map_err(|_| Error::Shutdown)
    }

    pub(crate) async fn start_bus_listener(
        &self,
        cookie: BusListenerCookie,
        scope: BusListenerScope,
    ) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::StartBusListener(StartBusListenerRequest {
                cookie,
                scope,
                reply: send,
            }))
            .map_err(|_| Error::Shutdown)?;

        let reply = recv.await.map_err(|_| Error::Shutdown)?;
        match reply {
            StartBusListenerResult::Ok => Ok(()),
            StartBusListenerResult::InvalidBusListener => Err(Error::InvalidBusListener),
            StartBusListenerResult::AlreadyStarted => Err(Error::BusListenerAlreadyStarted),
        }
    }

    pub(crate) async fn stop_bus_listener(&self, cookie: BusListenerCookie) -> Result<(), Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::StopBusListener(StopBusListenerRequest {
                cookie,
                reply: send,
            }))
            .map_err(|_| Error::Shutdown)?;

        let reply = recv.await.map_err(|_| Error::Shutdown)?;
        match reply {
            StopBusListenerResult::Ok => Ok(()),
            StopBusListenerResult::InvalidBusListener => Err(Error::InvalidBusListener),
            StopBusListenerResult::NotStarted => Err(Error::BusListenerNotStarted),
        }
    }

    /// Create a new `DiscovererBuilder`.
    pub fn create_discoverer<Key>(&self) -> DiscovererBuilder<Key> {
        Discoverer::builder(self)
    }

    /// Find an object with a specific set of services.
    ///
    /// If `object` is `None`, then any object that has all required services may be
    /// returned. Repeated calls to this function can return different objects.
    ///
    /// This is a convenience function for using a [`Discoverer`] to find a single object among all
    /// current objects on the bus.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin::core::{ObjectUuid, ServiceUuid};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
    /// # let client = broker.add_client().await;
    /// // Create an object and 2 services to find.
    /// let obj = client.create_object(ObjectUuid::new_v4()).await?;
    /// let svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await?;
    /// let svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await?;
    ///
    /// // Find the object.
    /// let (object_id, service_ids) = client
    ///     .find_object(Some(obj.id().uuid), &[svc1.id().uuid, svc2.id().uuid])
    ///     .await?
    ///     .unwrap();
    ///
    /// assert_eq!(object_id, obj.id());
    /// assert_eq!(service_ids[0], svc1.id());
    /// assert_eq!(service_ids[1], svc2.id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Without specifying an `ObjectUuid`:
    ///
    /// ```
    /// # use aldrin::core::{ObjectUuid, ServiceUuid};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio::TestBroker::new();
    /// # let client = broker.add_client().await;
    /// // Create 2 objects and sets of services to find.
    /// let obj1 = client.create_object(ObjectUuid::new_v4()).await?;
    /// let svc11 = obj1.create_service(ServiceUuid::new_v4(), 0).await?;
    /// let svc12 = obj1.create_service(ServiceUuid::new_v4(), 0).await?;
    ///
    /// let obj2 = client.create_object(ObjectUuid::new_v4()).await?;
    /// let svc21 = obj2.create_service(svc11.id().uuid, 0).await?;
    /// let svc22 = obj2.create_service(svc12.id().uuid, 0).await?;
    ///
    /// // Find any one of the objects above.
    /// let (object_id, service_ids) = client
    ///     .find_object(None, &[svc11.id().uuid, svc12.id().uuid])
    ///     .await?
    ///     .unwrap();
    ///
    /// assert!((object_id == obj1.id()) || (object_id == obj2.id()));
    /// assert!((service_ids[0] == svc11.id()) || (service_ids[0] == svc21.id()));
    /// assert!((service_ids[1] == svc12.id()) || (service_ids[1] == svc22.id()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_object<const N: usize>(
        &self,
        object: Option<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        let mut discoverer = self
            .create_discoverer()
            .object((), object, services.iter().copied())
            .build_current_only()
            .await?;

        let Some(event) = discoverer.next_event().await else {
            return Ok(None);
        };

        // SAFETY: This creates an array of MaybeUninit, which doesn't require initialization.
        let mut ids: [MaybeUninit<ServiceId>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for (&uuid, id) in services.iter().zip(&mut ids) {
            id.write(event.service_id(uuid));
        }

        // SAFETY: All N elements have been initialized in the loop above.
        //
        // In some future version of Rust, all this can be simplified; see:
        // https://github.com/rust-lang/rust/issues/96097
        // https://github.com/rust-lang/rust/issues/61956
        let ids = unsafe {
            (*(&MaybeUninit::new(ids) as *const _ as *const MaybeUninit<[ServiceId; N]>))
                .assume_init_read()
        };

        Ok(Some((event.object_id(), ids)))
    }

    /// Finds any object implementing a set of services.
    ///
    /// This is a shorthand for calling `find_object(None, services)`.
    pub async fn find_any_object<const N: usize>(
        &self,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        self.find_object(None, services).await
    }

    /// Finds a specific object implementing a set of services.
    ///
    /// This is a shorthand for calling `find_object(Some(object), services)`.
    pub async fn find_specific_object<const N: usize>(
        &self,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        self.find_object(Some(object.into()), services).await
    }

    /// Waits for an object with a specific set of services.
    ///
    /// If `object` is `None`, then any object that has all required services may be
    /// returned. Repeated calls to this function can return different objects.
    ///
    /// This is a convenience function for using a [`Discoverer`] to find a single object.
    pub async fn wait_for_object<const N: usize>(
        &self,
        object: Option<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        let mut discoverer = self
            .create_discoverer()
            .object((), object, services.iter().copied())
            .build()
            .await?;

        let Some(event) = discoverer.next_event().await else {
            return Err(Error::Shutdown);
        };

        // SAFETY: This creates an array of MaybeUninit, which doesn't require initialization.
        let mut ids: [MaybeUninit<ServiceId>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for (&uuid, id) in services.iter().zip(&mut ids) {
            id.write(event.service_id(uuid));
        }

        // SAFETY: All N elements have been initialized in the loop above.
        //
        // In some future version of Rust, all this can be simplified; see:
        // https://github.com/rust-lang/rust/issues/96097
        // https://github.com/rust-lang/rust/issues/61956
        let ids = unsafe {
            (*(&MaybeUninit::new(ids) as *const _ as *const MaybeUninit<[ServiceId; N]>))
                .assume_init_read()
        };

        Ok((event.object_id(), ids))
    }

    /// Wait for any object implementing a set of services.
    ///
    /// This is a shorthand for calling `wait_for_object(None, services)`.
    pub async fn wait_for_any_object<const N: usize>(
        &self,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        self.wait_for_object(None, services).await
    }

    /// Wait for a specific object implementing a set of services.
    ///
    /// This is a shorthand for calling `wait_for_object(Some(object), services)`.
    pub async fn wait_for_specific_object<const N: usize>(
        &self,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        self.wait_for_object(Some(object.into()), services).await
    }

    /// Creates a new lifetime scope.
    pub async fn create_lifetime_scope(&self) -> Result<LifetimeScope, Error> {
        self.create_object(ObjectUuid::new_v4())
            .await
            .map(LifetimeScope::new)
    }

    pub(crate) async fn create_lifetime_listener(&self) -> Result<LifetimeListener, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateLifetimeListener(reply))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    /// Create a [`Lifetime`] from an id.
    pub async fn create_lifetime(&self, id: LifetimeId) -> Result<Lifetime, Error> {
        Lifetime::create(self, id).await
    }

    /// Returns the protocol version that was negotiated with the broker.
    pub async fn version(&self) -> Result<ProtocolVersion, Error> {
        Ok(ProtocolVersion::V1_14)
    }

    /// Creates a new proxy to a service.
    pub async fn create_proxy(&self, id: ServiceId) -> Result<Proxy, Error> {
        Proxy::new(self.clone(), id).await
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

#[derive(Debug)]
pub(crate) struct CloseChannelEndFuture(oneshot::Receiver<Result<(), Error>>);

impl Future for CloseChannelEndFuture {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(Ok(Ok(()))) => Poll::Ready(Ok(())),
            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(Err(oneshot::Canceled)) => Poll::Ready(Err(Error::Shutdown)),
            Poll::Pending => Poll::Pending,
        }
    }
}
