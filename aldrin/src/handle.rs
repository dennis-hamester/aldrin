pub(crate) mod request;

use crate::discoverer::DiscovererBuilder;
use crate::lifetime::LifetimeListener;
use crate::low_level::{
    self, PendingReceiver, PendingReply, PendingSender, Proxy, ProxyId, Service, ServiceInfo,
    UnclaimedReceiver, UnclaimedSender,
};
use crate::{
    BusListener, ChannelBuilder, Discoverer, Error, Lifetime, LifetimeId, LifetimeScope, Object,
};
#[cfg(feature = "introspection")]
use aldrin_core::TypeId;
#[cfg(feature = "introspection")]
use aldrin_core::introspection::{DynIntrospectable, Introspectable, Introspection};
use aldrin_core::message::{
    AddBusListenerFilter, AddChannelCapacity, CallFunctionResult, ClearBusListenerFilters,
    DestroyBusListenerResult, DestroyObjectResult, RemoveBusListenerFilter, StartBusListenerResult,
    StopBusListenerResult,
};
use aldrin_core::tags::Tag;
use aldrin_core::{
    BusListenerCookie, BusListenerFilter, BusListenerScope, ChannelCookie, ChannelEnd,
    ObjectCookie, ObjectId, ObjectUuid, ProtocolVersion, Serialize, SerializedValue, ServiceId,
    ServiceUuid,
};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_channel::oneshot;
#[cfg(feature = "introspection")]
use request::QueryIntrospectionRequest;
use request::{
    CallFunctionReplyRequest, CallFunctionRequest, ClaimReceiverRequest, ClaimSenderRequest,
    CloseChannelEndRequest, CreateClaimedReceiverRequest, CreateObjectRequest, CreateProxyRequest,
    CreateServiceRequest, DestroyBusListenerRequest, DestroyObjectRequest, DestroyServiceRequest,
    EmitEventRequest, HandleRequest, SendItemRequest, StartBusListenerRequest,
    StopBusListenerRequest, SubscribeAllEventsRequest, SubscribeEventRequest,
    UnsubscribeAllEventsRequest, UnsubscribeEventRequest,
};
use std::hash::Hash;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

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
        Self { send }
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
        let _ = self.send.unbounded_send(HandleRequest::Shutdown);
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
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
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

        let _ = self
            .send
            .unbounded_send(HandleRequest::DestroyObject(DestroyObjectRequest {
                cookie,
                reply,
            }));
    }

    pub(crate) async fn create_service(
        &self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
        info: ServiceInfo,
    ) -> Result<Service, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::CreateService(CreateServiceRequest {
                object_id,
                service_uuid,
                info,
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

        let _ = self
            .send
            .unbounded_send(HandleRequest::DestroyService(DestroyServiceRequest {
                id,
                reply,
            }));
    }

    pub(crate) fn call<T: Tag>(
        &self,
        id: ServiceId,
        function: u32,
        args: impl Serialize<T>,
        version: Option<u32>,
    ) -> PendingReply {
        let (send, recv) = oneshot::channel();

        match SerializedValue::serialize_as(args) {
            Ok(value) => {
                let req = HandleRequest::CallFunction(CallFunctionRequest {
                    service_cookie: id.cookie,
                    function,
                    version,
                    value,
                    reply: send,
                });

                let _ = self.send.unbounded_send(req);
            }

            Err(e) => {
                let _ = send.send(Err(e.into()));
            }
        }

        PendingReply::new(recv, function, version)
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

    pub(crate) fn emit_event<T: Tag>(
        &self,
        service_id: ServiceId,
        event: u32,
        value: impl Serialize<T>,
    ) -> Result<(), Error> {
        let value = SerializedValue::serialize_as(value)?;
        self.send
            .unbounded_send(HandleRequest::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                value,
            }))
            .map_err(|_| Error::Shutdown)
    }

    /// Creates a low-level [`ChannelBuilder`](low_level::ChannelBuilder).
    ///
    /// Alternatively, [`ChannelBuilder::new`](low_level::ChannelBuilder::new) can be used as well.
    pub fn create_low_level_channel(&self) -> low_level::ChannelBuilder<'_> {
        low_level::ChannelBuilder::new(self)
    }

    /// Creates a [`ChannelBuilder`].
    ///
    /// Alternatively, [`ChannelBuilder::new`] can be used as well.
    pub fn create_channel<T>(&self) -> ChannelBuilder<'_, T> {
        ChannelBuilder::new(self)
    }

    pub(crate) async fn create_claimed_sender(
        &self,
    ) -> Result<(PendingSender, UnclaimedReceiver), Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::CreateClaimedSender(reply))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    pub(crate) async fn create_claimed_receiver(
        &self,
        capacity: u32,
    ) -> Result<(UnclaimedSender, PendingReceiver), Error> {
        let capacity = NonZeroU32::new(capacity).unwrap_or(NonZeroU32::new(1).unwrap());
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::CreateClaimedReceiver(
                CreateClaimedReceiverRequest { capacity, reply },
            ))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
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

    pub(crate) async fn claim_sender(
        &self,
        cookie: ChannelCookie,
    ) -> Result<(UnboundedReceiver<u32>, u32), Error> {
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
    ) -> Result<(UnboundedReceiver<SerializedValue>, NonZeroU32), Error> {
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
    /// Returns the timestamp when the [`Client`](crate::Client) has processed the request.
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
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
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
    pub async fn sync_client(&self) -> Result<Instant, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::SyncClient(reply))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    /// Synchronizes with the broker.
    ///
    /// Returns the timestamp when the [`Client`](crate::Client) has received the broker's reply.
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
    /// # use aldrin::low_level::ServiceInfo;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let obj = handle.create_object(ObjectUuid::new_v4()).await?;
    /// # let service = obj.create_service(ServiceUuid::new_v4(), ServiceInfo::new(0)).await?;
    ///
    /// service.emit(0, "Hi!")?;
    ///
    /// // Synchronize with the broker to ensure that the event has actually been processed.
    /// handle.sync_broker().await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync_broker(&self) -> Result<Instant, Error> {
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

        let _ = self.send.unbounded_send(HandleRequest::DestroyBusListener(
            DestroyBusListenerRequest { cookie, reply },
        ));
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
    pub fn create_discoverer<Key>(&self) -> DiscovererBuilder<'_, Key>
    where
        Key: Copy + Eq + Hash,
    {
        Discoverer::builder(self)
    }

    /// Finds an object with a specific set of services.
    ///
    /// If `object` is `None`, then any object that has all required services may be
    /// returned. Repeated calls to this function can return different objects.
    ///
    /// This is a convenience function for using a [`Discoverer`] to find a single object on the
    /// bus.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin::core::{ObjectUuid, ServiceUuid};
    /// # use aldrin::low_level::ServiceInfo;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let client = broker.add_client().await;
    /// // Create an object and 2 services to find.
    /// let obj = client.create_object(ObjectUuid::new_v4()).await?;
    /// let info = ServiceInfo::new(0);
    /// let svc1 = obj.create_service(ServiceUuid::new_v4(), info).await?;
    /// let svc2 = obj.create_service(ServiceUuid::new_v4(), info).await?;
    ///
    /// // Find the object.
    /// let (object_id, service_ids) = client
    ///     .find_object(Some(obj.id().uuid), [svc1.id().uuid, svc2.id().uuid])
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
    /// # use aldrin::low_level::ServiceInfo;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let client = broker.add_client().await;
    /// // Create 2 objects and sets of services to find.
    /// let obj1 = client.create_object(ObjectUuid::new_v4()).await?;
    /// let info = ServiceInfo::new(0);
    /// let svc11 = obj1.create_service(ServiceUuid::new_v4(), info).await?;
    /// let svc12 = obj1.create_service(ServiceUuid::new_v4(), info).await?;
    ///
    /// let obj2 = client.create_object(ObjectUuid::new_v4()).await?;
    /// let svc21 = obj2.create_service(svc11.id().uuid, info).await?;
    /// let svc22 = obj2.create_service(svc12.id().uuid, info).await?;
    ///
    /// // Find any one of the objects above.
    /// let (object_id, service_ids) = client
    ///     .find_object(None, [svc11.id().uuid, svc12.id().uuid])
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
    pub async fn find_object<S>(
        &self,
        object: Option<ObjectUuid>,
        services: S,
    ) -> Result<Option<(ObjectId, Vec<ServiceId>)>, Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        let services = services.into_iter().map(Into::into).collect::<Vec<_>>();

        let mut discoverer = self
            .create_discoverer()
            .add((), object, services.iter().copied())
            .build_current_only()
            .await?;

        Ok(discoverer
            .next_event()
            .await
            .map(|ev| (ev.object_id(), ev.service_ids(&discoverer, services))))
    }

    /// Finds an object with a specific set of services.
    ///
    /// This method is similar to [`find_object`](Self::find_object), but uses arrays for list of
    /// services.
    pub async fn find_object_n<const N: usize>(
        &self,
        object: Option<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        let mut discoverer = self
            .create_discoverer()
            .add((), object, services.iter().copied())
            .build_current_only()
            .await?;

        Ok(discoverer
            .next_event()
            .await
            .map(|ev| (ev.object_id(), ev.service_ids_n(&discoverer, services))))
    }

    /// Finds an object with a specific set of services.
    ///
    /// This is a shorthand for calling [`find_object`](Self::find_object) with `Some(object)`.
    pub async fn find_object_with_services<S>(
        &self,
        object: impl Into<ObjectUuid>,
        services: S,
    ) -> Result<Option<(ObjectId, Vec<ServiceId>)>, Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.find_object(Some(object.into()), services).await
    }

    /// Finds an object with a specific set of services.
    ///
    /// This is a shorthand for calling [`find_object_n`](Self::find_object_n) with `Some(object)`.
    pub async fn find_object_with_services_n<const N: usize>(
        &self,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        self.find_object_n(Some(object.into()), services).await
    }

    /// Finds a bare object without any associated services.
    ///
    /// This is a shorthand for calling [`find_object`](Self::find_object) with `Some(object)` and
    /// an empty set of services.
    pub async fn find_bare_object(
        &self,
        object: impl Into<ObjectUuid>,
    ) -> Result<Option<ObjectId>, Error> {
        self.find_object_n(Some(object.into()), &[])
            .await?
            .map(|(id, [])| Ok(id))
            .transpose()
    }

    /// Finds any object with a specific set of services.
    ///
    /// This is a shorthand for calling [`find_object`](Self::find_object) with `None` for the
    /// object.
    pub async fn find_any_object_with_services<S>(
        &self,
        services: S,
    ) -> Result<Option<(ObjectId, Vec<ServiceId>)>, Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.find_object(None, services).await
    }

    /// Finds any object with a specific set of services.
    ///
    /// This is a shorthand for calling [`find_object_n`](Self::find_object_n) with `None` for the
    /// object.
    pub async fn find_any_object_with_services_n<const N: usize>(
        &self,
        services: &[ServiceUuid; N],
    ) -> Result<Option<(ObjectId, [ServiceId; N])>, Error> {
        self.find_object_n(None, services).await
    }

    /// Waits for an object with a specific set of services.
    ///
    /// If `object` is `None`, then any object that has all required services may be
    /// returned. Repeated calls to this function can return different objects.
    ///
    /// This is a convenience function for using a [`Discoverer`] to wait for a single object on the
    /// bus.
    pub async fn wait_for_object<S>(
        &self,
        object: Option<ObjectUuid>,
        services: S,
    ) -> Result<(ObjectId, Vec<ServiceId>), Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        let services = services.into_iter().map(Into::into).collect::<Vec<_>>();

        let mut discoverer = self
            .create_discoverer()
            .add((), object, services.iter().copied())
            .build()
            .await?;

        discoverer
            .next_event()
            .await
            .map(|ev| (ev.object_id(), ev.service_ids(&discoverer, services)))
            .ok_or(Error::Shutdown)
    }

    /// Waits for an object with a specific set of services.
    ///
    /// This method is similar to [`wait_for_object`](Self::wait_for_object), but uses arrays for
    /// set of services.
    pub async fn wait_for_object_n<const N: usize>(
        &self,
        object: Option<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        let mut discoverer = self
            .create_discoverer()
            .add((), object, services.iter().copied())
            .build()
            .await?;

        discoverer
            .next_event()
            .await
            .map(|ev| (ev.object_id(), ev.service_ids_n(&discoverer, services)))
            .ok_or(Error::Shutdown)
    }

    /// Waits for an object with a specific set of services.
    ///
    /// This is a shorthand for calling [`wait_for_object`](Self::wait_for_object) with
    /// `Some(object)`.
    pub async fn wait_for_object_with_services<S>(
        &self,
        object: impl Into<ObjectUuid>,
        services: S,
    ) -> Result<(ObjectId, Vec<ServiceId>), Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.wait_for_object(Some(object.into()), services).await
    }

    /// Waits for an object with a specific set of services.
    ///
    /// This is a shorthand for calling [`wait_for_object_n`](Self::wait_for_object_n) with
    /// `Some(object)`.
    pub async fn wait_for_object_with_services_n<const N: usize>(
        &self,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        self.wait_for_object_n(Some(object.into()), services).await
    }

    /// Waits for a bare object without any associated services.
    ///
    /// This is a shorthand for calling [`wait_for_object`](Self::wait_for_object) with
    /// `Some(object)` and an empty set of services.
    pub async fn wait_for_bare_object(
        &self,
        object: impl Into<ObjectUuid>,
    ) -> Result<ObjectId, Error> {
        self.wait_for_object_n(Some(object.into()), &[])
            .await
            .map(|(id, [])| id)
    }

    /// Waits for any object with a specific set of services.
    ///
    /// This is a shorthand for calling [`wait_for_object`](Self::wait_for_object) with `None` for
    /// the object.
    pub async fn wait_for_any_object_with_services<S>(
        &self,
        services: S,
    ) -> Result<(ObjectId, Vec<ServiceId>), Error>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.wait_for_object(None, services).await
    }

    /// Waits for any object with a specific set of services.
    ///
    /// This is a shorthand for calling [`wait_for_object_n`](Self::wait_for_object_n) with `None`
    /// for the object.
    pub async fn wait_for_any_object_with_services_n<const N: usize>(
        &self,
        services: &[ServiceUuid; N],
    ) -> Result<(ObjectId, [ServiceId; N]), Error> {
        self.wait_for_object_n(None, services).await
    }

    /// Creates a new lifetime scope.
    pub async fn create_lifetime_scope(&self) -> Result<LifetimeScope, Error> {
        self.create_object(ObjectUuid::new_v4())
            .await
            .map(LifetimeScope::new_impl)
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
        Lifetime::new(self, id).await
    }

    /// Returns the protocol version that was negotiated with the broker.
    pub async fn version(&self) -> Result<ProtocolVersion, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::GetProtocolVersion(reply))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)
    }

    /// Creates a new proxy to a service.
    pub async fn create_proxy(&self, service: ServiceId) -> Result<Proxy, Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::CreateProxy(CreateProxyRequest {
                service,
                reply,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) fn destroy_proxy_now(&self, proxy: ProxyId) {
        let _ = self.send.unbounded_send(HandleRequest::DestroyProxy(proxy));
    }

    pub(crate) async fn subscribe_event(&self, proxy: ProxyId, event: u32) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::SubscribeEvent(SubscribeEventRequest {
                proxy,
                event,
                reply,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) async fn unsubscribe_event(&self, proxy: ProxyId, event: u32) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::UnsubscribeEvent(UnsubscribeEventRequest {
                proxy,
                event,
                reply,
            }))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) async fn subscribe_all_events(&self, proxy: ProxyId) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::SubscribeAllEvents(
                SubscribeAllEventsRequest { proxy, reply },
            ))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    pub(crate) async fn unsubscribe_all_events(&self, proxy: ProxyId) -> Result<(), Error> {
        let (reply, recv) = oneshot::channel();

        self.send
            .unbounded_send(HandleRequest::UnsubscribeAllEvents(
                UnsubscribeAllEventsRequest { proxy, reply },
            ))
            .map_err(|_| Error::Shutdown)?;

        recv.await.map_err(|_| Error::Shutdown)?
    }

    /// Registers an introspectable type with the client.
    ///
    /// Registered types are made available to be queried by other clients.
    #[cfg(feature = "introspection")]
    pub fn register_introspection<T: Introspectable + ?Sized>(&self) -> Result<(), Error> {
        self.register_introspection_dyn(DynIntrospectable::new::<T>())
    }

    /// Registers an introspectable type with the client.
    ///
    /// Registered types are made available to be queried by other clients.
    #[cfg(feature = "introspection")]
    pub fn register_introspection_dyn(&self, ty: DynIntrospectable) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::RegisterIntrospection(ty))
            .map_err(|_| Error::Shutdown)
    }

    /// Submits all registered introspectable types to the broker.
    #[cfg(feature = "introspection")]
    pub fn submit_introspection(&self) -> Result<(), Error> {
        self.send
            .unbounded_send(HandleRequest::SubmitIntrospection)
            .map_err(|_| Error::Shutdown)
    }

    /// Queries the introspection for a type.
    #[cfg(feature = "introspection")]
    pub async fn query_introspection(
        &self,
        type_id: TypeId,
    ) -> Result<Option<Introspection>, Error> {
        let (reply, recv) = oneshot::channel();
        self.send
            .unbounded_send(HandleRequest::QueryIntrospection(
                QueryIntrospectionRequest { type_id, reply },
            ))
            .map_err(|_| Error::Shutdown)?;

        match recv.await {
            Ok(Some(introspection)) => Ok(introspection.deserialize().ok()),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::Shutdown),
        }
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        let _ = self.send.unbounded_send(HandleRequest::HandleCloned);

        Self {
            send: self.send.clone(),
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.send.unbounded_send(HandleRequest::HandleDropped);
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
