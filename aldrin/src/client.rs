mod proxies;
mod select;

use crate::bus_listener::{BusListener, BusListenerHandle};
use crate::channel::{
    PendingReceiverInner, PendingSenderInner, ReceiverInner, SenderInner, UnclaimedReceiverInner,
    UnclaimedSenderInner,
};
#[cfg(feature = "introspection")]
use crate::core::introspection::Introspection;
use crate::core::message::{
    AbortFunctionCall, AddBusListenerFilter, AddChannelCapacity, BusListenerCurrentFinished,
    CallFunction, CallFunctionReply, CallFunctionResult, ChannelEndClaimed, ChannelEndClosed,
    ClaimChannelEnd, ClaimChannelEndReply, ClaimChannelEndResult, ClearBusListenerFilters,
    CloseChannelEnd, CloseChannelEndReply, CloseChannelEndResult, Connect2, ConnectData,
    ConnectResult, CreateBusListener, CreateBusListenerReply, CreateChannel, CreateChannelReply,
    CreateObject, CreateObjectReply, CreateObjectResult, CreateService, CreateService2,
    CreateServiceReply, CreateServiceResult, DestroyBusListener, DestroyBusListenerReply,
    DestroyBusListenerResult, DestroyObject, DestroyObjectReply, DestroyObjectResult,
    DestroyService, DestroyServiceReply, DestroyServiceResult, EmitBusEvent, EmitEvent,
    ItemReceived, Message, QueryIntrospection, QueryIntrospectionReply, QueryIntrospectionResult,
    QueryServiceInfo, QueryServiceInfoReply, QueryServiceInfoResult, QueryServiceVersion,
    QueryServiceVersionReply, QueryServiceVersionResult, RemoveBusListenerFilter, SendItem,
    ServiceDestroyed, Shutdown, StartBusListener, StartBusListenerReply, StartBusListenerResult,
    StopBusListener, StopBusListenerReply, StopBusListenerResult, SubscribeEvent,
    SubscribeEventReply, SubscribeEventResult, Sync, SyncReply, UnsubscribeEvent,
};
use crate::core::transport::{AsyncTransport, AsyncTransportExt};
#[cfg(feature = "introspection")]
use crate::core::TypeId;
use crate::core::{
    BusListenerCookie, ChannelCookie, ChannelEnd, ChannelEndWithCapacity, Deserialize, ObjectId,
    ProtocolVersion, Serialize, SerializedValue, SerializedValueSlice, ServiceCookie, ServiceId,
    ServiceInfo,
};
use crate::error::{ConnectError, RunError};
use crate::function_call_map::FunctionCallMap;
use crate::handle::request::{
    CallFunctionReplyRequest, CallFunctionRequest, ClaimReceiverRequest, ClaimSenderRequest,
    CloseChannelEndRequest, CreateBusListenerRequest, CreateClaimedReceiverRequest,
    CreateClaimedSenderRequest, CreateLifetimeListenerRequest, CreateObjectRequest,
    CreateProxyRequest, CreateServiceRequest, DestroyBusListenerRequest, DestroyObjectRequest,
    DestroyServiceRequest, EmitEventRequest, HandleRequest, SendItemRequest,
    StartBusListenerRequest, StopBusListenerRequest, SubscribeEventRequest, SyncBrokerRequest,
    SyncClientRequest, UnsubscribeEventRequest,
};
#[cfg(feature = "introspection")]
use crate::handle::request::{IntrospectionQueryResult, QueryIntrospectionRequest};
use crate::lifetime::LifetimeListener;
use crate::low_level::{ProxyId, RawCall, Service};
use crate::serial_map::SerialMap;
use crate::{Error, Handle, Object};
use futures_channel::{mpsc, oneshot};
use proxies::{Proxies, SubscribeResult};
use select::{Select, Selected};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::mem;

const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V1_17;

/// Aldrin client used to connect to a broker.
///
/// This is the first entry point to Aldrin. A [`Client`] is used to establish a connection to an
/// Aldrin broker. Afterwards, it should be turned into a [`Future`](std::future::Future) with the
/// [`run`](Client::run) method, which must then be continuously polled and run to completion.
///
/// All interaction with a [`Client`] happens asynchronously through one or more
/// [`Handle`s](Handle), which must be acquired with [`Client::handle`] before calling
/// [`Client::run`].
///
/// # Shutdown
///
/// A [`Client`] will automatically shut down when the last [`Handle`] has been dropped. Keep in
/// mind that several other types (such as e.g. [`Object`]) keep an internal [`Handle`]. Use
/// [`Handle::shutdown`] to shut down the [`Client`] manually.
#[derive(Debug)]
#[must_use = "clients do nothing unless you `.await` or poll `Client::run()`"]
pub struct Client<T>
where
    T: AsyncTransport + Unpin,
{
    select: Select,
    t: T,
    protocol_version: ProtocolVersion,
    recv: mpsc::UnboundedReceiver<HandleRequest>,
    handle: Handle,
    num_handles: usize,
    create_object: SerialMap<CreateObjectRequest>,
    destroy_object: SerialMap<oneshot::Sender<DestroyObjectResult>>,
    create_service: SerialMap<CreateServiceRequest>,
    destroy_service: SerialMap<DestroyServiceRequest>,
    function_calls: FunctionCallMap,
    services: HashMap<ServiceCookie, mpsc::UnboundedSender<RawCall>>,
    broker_subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
    create_channel: SerialMap<CreateChannelData>,
    close_channel_end: SerialMap<CloseChannelEndRequest>,
    claim_channel_end: SerialMap<ClaimChannelEndData>,
    senders: HashMap<ChannelCookie, SenderState>,
    receivers: HashMap<ChannelCookie, ReceiverState>,
    sync: SerialMap<SyncBrokerRequest>,
    create_bus_listener: SerialMap<CreateBusListenerData>,
    destroy_bus_listener: SerialMap<DestroyBusListenerRequest>,
    start_bus_listener: SerialMap<StartBusListenerRequest>,
    stop_bus_listener: SerialMap<StopBusListenerRequest>,
    bus_listeners: HashMap<BusListenerCookie, BusListenerHandle>,
    abort_call_handles: HashMap<u32, oneshot::Sender<()>>,
    query_service_info: SerialMap<CreateProxyRequest>,
    query_service_version: SerialMap<CreateProxyRequest>,
    subscribe_event: SerialMap<SubscribeEventRequest>,
    proxies: Proxies,
    #[cfg(feature = "introspection")]
    introspection: HashMap<TypeId, &'static Introspection>,
    #[cfg(feature = "introspection")]
    query_introspection: SerialMap<QueryIntrospectionRequest>,
}

impl<T> Client<T>
where
    T: AsyncTransport + Unpin,
{
    /// Creates a client and connects to an Aldrin broker.
    ///
    /// If you need to send custom data to the broker, then use
    /// [`connect_with_data`](Self::connect_with_data) instead. This function sends `()` and
    /// discards the broker's data.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
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
    /// // Create an AsyncTransport for connecting to the broker.
    /// // let async_transport = ...
    ///
    /// // Connect to the broker:
    /// let client = Client::connect(async_transport).await?;
    /// # tokio::spawn(conn.await??.run());
    ///
    /// // Acquire a handle and spawn the client:
    /// let handle = client.handle().clone();
    /// let join = tokio::spawn(client.run());
    ///
    /// // The client is now fully connected and can be interacted with through the handle.
    ///
    /// // Shut down client:
    /// handle.shutdown();
    /// join.await??;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(t: T) -> Result<Self, ConnectError<T::Error>> {
        let (client, _) = Self::connect_with_data::<()>(t, None).await?;
        Ok(client)
    }

    /// Creates a client and connects to an Aldrin broker. Allows to send and receive custom data.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
    pub async fn connect_with_data<D: Serialize + ?Sized>(
        mut t: T,
        data: Option<&D>,
    ) -> Result<(Self, Option<SerializedValue>), ConnectError<T::Error>> {
        let mut connect_data = ConnectData::new();

        if let Some(data) = data {
            connect_data.serialize_user(data)?;
        }

        let connect = Connect2::with_serialize_data(
            PROTOCOL_VERSION.major(),
            PROTOCOL_VERSION.minor(),
            &connect_data,
        )?;

        t.send_and_flush(connect)
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = match t.receive().await.map_err(ConnectError::Transport)? {
            Message::ConnectReply2(connect_reply) => connect_reply,
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg)),
        };

        let connect_reply_data = connect_reply.deserialize_connect_data()?;

        let minor_version = match connect_reply.result {
            ConnectResult::Ok(minor_version) => minor_version,
            ConnectResult::Rejected => return Err(ConnectError::Rejected(connect_reply_data.user)),
            ConnectResult::IncompatibleVersion => return Err(ConnectError::IncompatibleVersion),
        };

        let protocol_version = ProtocolVersion::new(PROTOCOL_VERSION.major(), minor_version)
            .map_err(|_| ConnectError::IncompatibleVersion)?;

        if protocol_version > PROTOCOL_VERSION {
            return Err(ConnectError::IncompatibleVersion);
        }

        let (send, recv) = mpsc::unbounded();
        let client = Client {
            select: Select::new(),
            t,
            protocol_version,
            recv,
            handle: Handle::new(send),
            num_handles: 1,
            create_object: SerialMap::new(),
            destroy_object: SerialMap::new(),
            create_service: SerialMap::new(),
            destroy_service: SerialMap::new(),
            function_calls: FunctionCallMap::new(),
            services: HashMap::new(),
            broker_subscriptions: HashMap::new(),
            create_channel: SerialMap::new(),
            close_channel_end: SerialMap::new(),
            claim_channel_end: SerialMap::new(),
            senders: HashMap::new(),
            receivers: HashMap::new(),
            sync: SerialMap::new(),
            create_bus_listener: SerialMap::new(),
            destroy_bus_listener: SerialMap::new(),
            start_bus_listener: SerialMap::new(),
            stop_bus_listener: SerialMap::new(),
            bus_listeners: HashMap::new(),
            abort_call_handles: HashMap::new(),
            query_service_info: SerialMap::new(),
            query_service_version: SerialMap::new(),
            subscribe_event: SerialMap::new(),
            proxies: Proxies::new(),
            #[cfg(feature = "introspection")]
            introspection: HashMap::new(),
            #[cfg(feature = "introspection")]
            query_introspection: SerialMap::new(),
        };

        Ok((client, connect_reply_data.user))
    }

    /// Creates a client and connects to an Aldrin broker. Allows to send and receive custom data.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
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
    /// # tokio::spawn(async move { handle.connect(t2).await });
    /// // Create an AsyncTransport for connecting to the broker.
    /// // let async_transport = ...
    ///
    /// // Connect to the broker, sending some custom data.
    /// let (client, data) = Client::connect_with_data(async_transport, Some("Hi!")).await?;
    ///
    /// println!("Data the broker sent back: {:?}.", data);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_data_and_deserialize<D1, D2>(
        t: T,
        data: Option<&D1>,
    ) -> Result<(Self, Option<D2>), ConnectError<T::Error>>
    where
        D1: Serialize + ?Sized,
        D2: Deserialize,
    {
        let (client, data) = Self::connect_with_data(t, data).await?;
        let data = data
            .as_deref()
            .map(SerializedValueSlice::deserialize)
            .transpose()?;

        Ok((client, data))
    }

    /// Returns a handle to the client.
    ///
    /// After creating the [`Client`], [`Handle`s](Handle) are the primary entry point for
    /// interacting with it.
    ///
    /// When the last [`Handle`] is dropped, the [`Client`] will automatically shut down.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Returns the protocol version that was negotiated with the broker.
    pub fn protocol_version(self) -> ProtocolVersion {
        self.protocol_version
    }

    /// Runs the client until it shuts down.
    ///
    /// After creating a [`Client`] it is important to run it before calling any method on a
    /// [`Handle`].
    ///
    /// This is a long running method, that will only complete once the [`Client`] has shut down. It
    /// should ideally be spawned on a dedicated task (not for performance or technical reasons, but
    /// for ergonomics).
    ///
    /// # Shutdown
    ///
    /// A running [`Client`] can be shut down manually with [`Handle::shutdown`]. It will also
    /// automatically shut down when the last [`Handle`] has been dropped. Be aware, that some
    /// types (such as e.g. [`Service`]) hold an internal [`Handle`] and will thus keep the
    /// [`Client`] running. [`Client`s](Client) can also be instructed by the broker to shut down.
    pub async fn run(mut self) -> Result<(), RunError<T::Error>> {
        loop {
            match self.select().await {
                Selected::Transport(Ok(Message::Shutdown(Shutdown))) => {
                    self.t.send_and_flush(Shutdown).await?;
                    return Ok(());
                }

                Selected::Transport(Ok(msg)) => self.handle_message(msg).await?,
                Selected::Transport(Err(e)) => return Err(e.into()),
                Selected::Handle(HandleRequest::Shutdown) => break,
                Selected::Handle(req) => self.handle_request(req).await?,
                Selected::AbortFunctionCall(serial) => self.abort_function_call(serial).await?,
            }

            if self.num_handles == 1 {
                break;
            }
        }

        self.t.send_and_flush(Shutdown).await?;
        self.drain_transport().await?;
        Ok(())
    }

    async fn select(&mut self) -> Selected<T> {
        self.select
            .select(&mut self.t, &mut self.recv, &mut self.function_calls)
            .await
    }

    async fn drain_transport(&mut self) -> Result<(), RunError<T::Error>> {
        loop {
            if let Message::Shutdown(Shutdown) = self.t.receive().await? {
                return Ok(());
            }
        }
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), RunError<T::Error>> {
        match msg {
            Message::CreateObjectReply(msg) => self.msg_create_object_reply(msg)?,
            Message::DestroyObjectReply(msg) => self.msg_destroy_object_reply(msg),
            Message::CreateServiceReply(msg) => self.msg_create_service_reply(msg)?,
            Message::DestroyServiceReply(msg) => self.msg_destroy_service_reply(msg),
            Message::CallFunction(msg) => self.msg_call_function(msg).await?,
            Message::CallFunctionReply(msg) => self.msg_call_function_reply(msg),
            Message::SubscribeEvent(msg) => self.msg_subscribe_event(msg),
            Message::UnsubscribeEvent(msg) => self.msg_unsubscribe_event(msg),
            Message::CreateChannelReply(msg) => self.msg_create_channel_reply(msg)?,
            Message::CloseChannelEndReply(msg) => self.msg_close_channel_end_reply(msg)?,
            Message::ChannelEndClosed(msg) => self.msg_channel_end_closed(msg)?,
            Message::ClaimChannelEndReply(msg) => self.msg_claim_channel_end_reply(msg)?,
            Message::ChannelEndClaimed(msg) => self.msg_channel_end_claimed(msg)?,
            Message::ItemReceived(msg) => self.msg_item_received(msg)?,
            Message::AddChannelCapacity(msg) => self.msg_add_channel_capacity(msg)?,
            Message::SyncReply(msg) => self.msg_sync_reply(msg)?,
            Message::CreateBusListenerReply(msg) => self.msg_create_bus_listener_reply(msg)?,
            Message::DestroyBusListenerReply(msg) => self.msg_destroy_bus_listener_reply(msg)?,
            Message::StartBusListenerReply(msg) => self.msg_start_bus_listener_reply(msg)?,
            Message::StopBusListenerReply(msg) => self.msg_stop_bus_listener_reply(msg)?,
            Message::EmitBusEvent(msg) => self.msg_emit_bus_event(msg)?,
            Message::BusListenerCurrentFinished(msg) => {
                self.msg_bus_listener_current_finished(msg)?
            }
            Message::AbortFunctionCall(msg) => self.msg_abort_function_call(msg)?,
            Message::QueryIntrospection(msg) => self.msg_query_introspection(msg).await?,
            Message::QueryIntrospectionReply(msg) => self.msg_query_introspection_reply(msg)?,
            Message::QueryServiceInfoReply(msg) => self.msg_query_service_info_reply(msg)?,
            Message::QueryServiceVersionReply(msg) => self.msg_query_service_version_reply(msg)?,
            Message::SubscribeEventReply(msg) => self.msg_subscribe_event_reply(msg)?,
            Message::EmitEvent(msg) => self.msg_emit_event(msg),
            Message::ServiceDestroyed(msg) => self.msg_service_destroyed(msg),

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObject(_)
            | Message::DestroyObject(_)
            | Message::CreateService(_)
            | Message::DestroyService(_)
            | Message::QueryServiceVersion(_)
            | Message::CreateChannel(_)
            | Message::CloseChannelEnd(_)
            | Message::ClaimChannelEnd(_)
            | Message::SendItem(_)
            | Message::Sync(_)
            | Message::CreateBusListener(_)
            | Message::DestroyBusListener(_)
            | Message::AddBusListenerFilter(_)
            | Message::RemoveBusListenerFilter(_)
            | Message::ClearBusListenerFilters(_)
            | Message::StartBusListener(_)
            | Message::StopBusListener(_)
            | Message::Connect2(_)
            | Message::ConnectReply2(_)
            | Message::RegisterIntrospection(_)
            | Message::CreateService2(_)
            | Message::QueryServiceInfo(_) => return Err(RunError::UnexpectedMessageReceived(msg)),

            Message::Shutdown(Shutdown) => unreachable!(), // Handled in run.
        }

        Ok(())
    }

    fn msg_create_object_reply(
        &mut self,
        msg: CreateObjectReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.create_object.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        let reply = match msg.result {
            CreateObjectResult::Ok(cookie) => Ok(Object::new_impl(
                ObjectId::new(req.uuid, cookie),
                self.handle.clone(),
            )),

            CreateObjectResult::DuplicateObject => Err(Error::DuplicateObject),
        };

        let _ = req.reply.send(reply);
        Ok(())
    }

    fn msg_destroy_object_reply(&mut self, msg: DestroyObjectReply) {
        if let Some(send) = self.destroy_object.remove(msg.serial) {
            let _ = send.send(msg.result);
        }
    }

    fn msg_create_service_reply(
        &mut self,
        msg: CreateServiceReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.create_service.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        let reply = match msg.result {
            CreateServiceResult::Ok(cookie) => {
                let (send, function_calls) = mpsc::unbounded();
                let dup = self.services.insert(cookie, send);
                debug_assert!(dup.is_none());

                Ok(Service::new_impl(
                    ServiceId::new(req.object_id, req.service_uuid, cookie),
                    req.info,
                    self.handle.clone(),
                    function_calls,
                ))
            }

            CreateServiceResult::DuplicateService => Err(Error::DuplicateService),
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject),
            CreateServiceResult::ForeignObject => unreachable!(),
        };

        let _ = req.reply.send(reply);
        Ok(())
    }

    fn msg_destroy_service_reply(&mut self, msg: DestroyServiceReply) {
        let Some(req) = self.destroy_service.remove(msg.serial) else {
            return;
        };

        let reply = match msg.result {
            DestroyServiceResult::Ok => {
                let contained = self.services.remove(&req.id.cookie);
                debug_assert!(contained.is_some());
                self.broker_subscriptions.remove(&req.id.cookie);
                Ok(())
            }

            DestroyServiceResult::InvalidService => Err(Error::InvalidService),
            DestroyServiceResult::ForeignObject => unreachable!(),
        };

        let _ = req.reply.send(reply);
    }

    async fn msg_call_function(&mut self, msg: CallFunction) -> Result<(), RunError<T::Error>> {
        let send = self
            .services
            .get_mut(&msg.service_cookie)
            .expect("inconsistent state");

        let (abort_send, abort_recv) = oneshot::channel();

        let req = RawCall {
            serial: msg.serial,
            function: msg.function,
            args: msg.value,
            aborted: abort_recv,
        };

        if send.unbounded_send(req).is_ok() {
            let dup = self.abort_call_handles.insert(msg.serial, abort_send);
            assert!(dup.is_none());
        } else {
            self.t
                .send_and_flush(CallFunctionReply {
                    serial: msg.serial,
                    result: CallFunctionResult::InvalidService,
                })
                .await?;
        }

        Ok(())
    }

    fn msg_call_function_reply(&mut self, msg: CallFunctionReply) {
        if let Some(send) = self.function_calls.remove(msg.serial) {
            let _ = send.send(Ok(msg.result));
        }
    }

    fn msg_subscribe_event(&mut self, msg: SubscribeEvent) {
        self.broker_subscriptions
            .entry(msg.service_cookie)
            .or_default()
            .insert(msg.event);
    }

    fn msg_unsubscribe_event(&mut self, msg: UnsubscribeEvent) {
        let Entry::Occupied(mut subs) = self.broker_subscriptions.entry(msg.service_cookie) else {
            return;
        };

        subs.get_mut().remove(&msg.event);
        if subs.get().is_empty() {
            subs.remove();
        }
    }

    fn msg_create_channel_reply(
        &mut self,
        msg: CreateChannelReply,
    ) -> Result<(), RunError<T::Error>> {
        match self.create_channel.remove(msg.serial) {
            Some(CreateChannelData::Sender(reply)) => {
                let (send, recv) = oneshot::channel();
                let sender = PendingSenderInner::new(msg.cookie, self.handle.clone(), recv);
                let receiver = UnclaimedReceiverInner::new(msg.cookie, self.handle.clone());
                let dup = self.senders.insert(msg.cookie, SenderState::Pending(send));
                debug_assert!(dup.is_none());
                let _ = reply.send((sender, receiver));
                Ok(())
            }

            Some(CreateChannelData::Receiver(req)) => {
                let (send, recv) = oneshot::channel();
                let sender = UnclaimedSenderInner::new(msg.cookie, self.handle.clone());
                let receiver =
                    PendingReceiverInner::new(msg.cookie, self.handle.clone(), recv, req.capacity);
                let dup = self
                    .receivers
                    .insert(msg.cookie, ReceiverState::Pending(send));
                debug_assert!(dup.is_none());
                let _ = req.reply.send((sender, receiver));
                Ok(())
            }

            None => Err(RunError::UnexpectedMessageReceived(msg.into())),
        }
    }

    fn msg_close_channel_end_reply(
        &mut self,
        msg: CloseChannelEndReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.close_channel_end.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        if req.claimed {
            match req.end {
                ChannelEnd::Sender => {
                    let contained = self.senders.remove(&req.cookie);
                    debug_assert!(contained.is_some());
                }

                ChannelEnd::Receiver => {
                    let contained = self.receivers.remove(&req.cookie);
                    debug_assert!(contained.is_some());
                }
            }
        }

        let res = match msg.result {
            CloseChannelEndResult::Ok => Ok(()),

            CloseChannelEndResult::InvalidChannel | CloseChannelEndResult::ForeignChannel => {
                Err(Error::InvalidChannel)
            }
        };

        let _ = req.reply.send(res);
        Ok(())
    }

    fn msg_channel_end_closed(&mut self, msg: ChannelEndClosed) -> Result<(), RunError<T::Error>> {
        match msg.end {
            ChannelEnd::Sender => {
                let receiver = self
                    .receivers
                    .get_mut(&msg.cookie)
                    .map(|receiver| mem::replace(receiver, ReceiverState::SenderClosed));

                match receiver {
                    Some(ReceiverState::Pending(_)) | Some(ReceiverState::Established(_)) => Ok(()),

                    Some(ReceiverState::SenderClosed) | None => {
                        Err(RunError::UnexpectedMessageReceived(msg.into()))
                    }
                }
            }

            ChannelEnd::Receiver => {
                let sender = self
                    .senders
                    .get_mut(&msg.cookie)
                    .map(|sender| mem::replace(sender, SenderState::ReceiverClosed));

                match sender {
                    Some(SenderState::Pending(_)) | Some(SenderState::Established(_)) => Ok(()),

                    Some(SenderState::ReceiverClosed) | None => {
                        Err(RunError::UnexpectedMessageReceived(msg.into()))
                    }
                }
            }
        }
    }

    fn msg_claim_channel_end_reply(
        &mut self,
        msg: ClaimChannelEndReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.claim_channel_end.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        match req {
            ClaimChannelEndData::Sender(req) => match msg.result {
                ClaimChannelEndResult::SenderClaimed(capacity) => {
                    let (send, recv) = mpsc::unbounded();
                    let dup = self
                        .senders
                        .insert(req.cookie, SenderState::Established(send));
                    debug_assert!(dup.is_none());
                    let sender = SenderInner::new(req.cookie, self.handle.clone(), recv, capacity);
                    let _ = req.reply.send(Ok(sender));
                }

                ClaimChannelEndResult::ReceiverClaimed => {
                    return Err(RunError::UnexpectedMessageReceived(msg.into()))
                }

                ClaimChannelEndResult::InvalidChannel | ClaimChannelEndResult::AlreadyClaimed => {
                    let _ = req.reply.send(Err(Error::InvalidChannel));
                }
            },

            ClaimChannelEndData::Receiver(req) => match msg.result {
                ClaimChannelEndResult::SenderClaimed(_) => {
                    return Err(RunError::UnexpectedMessageReceived(msg.into()))
                }

                ClaimChannelEndResult::ReceiverClaimed => {
                    let (send, recv) = mpsc::unbounded();
                    let dup = self
                        .receivers
                        .insert(req.cookie, ReceiverState::Established(send));
                    debug_assert!(dup.is_none());
                    let receiver =
                        ReceiverInner::new(req.cookie, self.handle.clone(), recv, req.capacity);
                    let _ = req.reply.send(Ok(receiver));
                }

                ClaimChannelEndResult::InvalidChannel | ClaimChannelEndResult::AlreadyClaimed => {
                    let _ = req.reply.send(Err(Error::InvalidChannel));
                }
            },
        }

        Ok(())
    }

    fn msg_channel_end_claimed(
        &mut self,
        msg: ChannelEndClaimed,
    ) -> Result<(), RunError<T::Error>> {
        match msg.end {
            ChannelEndWithCapacity::Sender => {
                let Some(receiver) = self.receivers.get_mut(&msg.cookie) else {
                    return Err(RunError::UnexpectedMessageReceived(msg.into()));
                };

                let (send, recv) = mpsc::unbounded();

                match mem::replace(receiver, ReceiverState::Established(send)) {
                    ReceiverState::Pending(send) => {
                        let _ = send.send(recv);
                        Ok(())
                    }

                    ReceiverState::Established(_) | ReceiverState::SenderClosed => {
                        Err(RunError::UnexpectedMessageReceived(msg.into()))
                    }
                }
            }

            ChannelEndWithCapacity::Receiver(capacity) => {
                let Some(sender) = self.senders.get_mut(&msg.cookie) else {
                    return Err(RunError::UnexpectedMessageReceived(msg.into()));
                };

                let (send, recv) = mpsc::unbounded();

                match mem::replace(sender, SenderState::Established(send)) {
                    SenderState::Pending(send) => {
                        let _ = send.send((capacity, recv));
                        Ok(())
                    }

                    SenderState::Established(_) | SenderState::ReceiverClosed => {
                        Err(RunError::UnexpectedMessageReceived(msg.into()))
                    }
                }
            }
        }
    }

    fn msg_item_received(&self, msg: ItemReceived) -> Result<(), RunError<T::Error>> {
        if let Some(ReceiverState::Established(send)) = self.receivers.get(&msg.cookie) {
            let _ = send.unbounded_send(msg.value);
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    fn msg_add_channel_capacity(&self, msg: AddChannelCapacity) -> Result<(), RunError<T::Error>> {
        if let Some(SenderState::Established(send)) = self.senders.get(&msg.cookie) {
            let _ = send.unbounded_send(msg.capacity);
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    fn msg_sync_reply(&mut self, msg: SyncReply) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.sync.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        let _ = req.send(());
        Ok(())
    }

    fn msg_create_bus_listener_reply(
        &mut self,
        msg: CreateBusListenerReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(data) = self.create_bus_listener.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        let (send, recv) = mpsc::unbounded();

        match data {
            CreateBusListenerData::BusListener(reply) => {
                let listener = BusListener::new_impl(msg.cookie, self.handle.clone(), recv);
                let _ = reply.send(listener);
            }

            CreateBusListenerData::LifetimeListener(reply) => {
                let listener = LifetimeListener::new(msg.cookie, self.handle.clone(), recv);
                let _ = reply.send(listener);
            }
        }

        let bus_listener_handle = BusListenerHandle::new(send);
        let dup = self.bus_listeners.insert(msg.cookie, bus_listener_handle);
        assert!(dup.is_none());

        Ok(())
    }

    fn msg_destroy_bus_listener_reply(
        &mut self,
        msg: DestroyBusListenerReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.destroy_bus_listener.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        if msg.result == DestroyBusListenerResult::Ok {
            let contained = self.bus_listeners.remove(&req.cookie);
            debug_assert!(contained.is_some());
        }

        let _ = req.reply.send(msg.result);

        Ok(())
    }

    fn msg_start_bus_listener_reply(
        &mut self,
        msg: StartBusListenerReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.start_bus_listener.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        if msg.result == StartBusListenerResult::Ok {
            let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
                return Err(RunError::UnexpectedMessageReceived(msg.into()));
            };

            if bus_listener.start(req.scope) {
                let _ = req.reply.send(msg.result);
                Ok(())
            } else {
                Err(RunError::UnexpectedMessageReceived(msg.into()))
            }
        } else {
            let _ = req.reply.send(msg.result);
            Ok(())
        }
    }

    fn msg_stop_bus_listener_reply(
        &mut self,
        msg: StopBusListenerReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.stop_bus_listener.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        if msg.result == StopBusListenerResult::Ok {
            let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
                return Err(RunError::UnexpectedMessageReceived(msg.into()));
            };

            if bus_listener.stop() {
                let _ = req.reply.send(msg.result);
                Ok(())
            } else {
                Err(RunError::UnexpectedMessageReceived(msg.into()))
            }
        } else {
            let _ = req.reply.send(msg.result);
            Ok(())
        }
    }

    fn msg_emit_bus_event(&self, msg: EmitBusEvent) -> Result<(), RunError<T::Error>> {
        if let Some(cookie) = msg.cookie {
            let Some(bus_listener) = self.bus_listeners.get(&cookie) else {
                return Err(RunError::UnexpectedMessageReceived(msg.into()));
            };

            if bus_listener.emit_current(msg.event) {
                Ok(())
            } else {
                Err(RunError::UnexpectedMessageReceived(msg.into()))
            }
        } else {
            for bus_listener in self.bus_listeners.values() {
                bus_listener.emit_new_if_matches(msg.event);
            }

            Ok(())
        }
    }

    fn msg_bus_listener_current_finished(
        &mut self,
        msg: BusListenerCurrentFinished,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&msg.cookie) {
            if bus_listener.current_finished() {
                Ok(())
            } else {
                Err(RunError::UnexpectedMessageReceived(msg.into()))
            }
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    fn msg_abort_function_call(
        &mut self,
        msg: AbortFunctionCall,
    ) -> Result<(), RunError<T::Error>> {
        if self.protocol_version >= ProtocolVersion::V1_16 {
            self.abort_call_handles.remove(&msg.serial);
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    #[cfg(feature = "introspection")]
    async fn msg_query_introspection(
        &mut self,
        msg: QueryIntrospection,
    ) -> Result<(), RunError<T::Error>> {
        if self.protocol_version >= ProtocolVersion::V1_17 {
            let reply = if let Some(introspection) = self.introspection.get(&msg.type_id) {
                QueryIntrospectionReply::ok_with_serialize_introspection(msg.serial, introspection)
                    .map_err(RunError::Serialize)?
            } else {
                QueryIntrospectionReply {
                    serial: msg.serial,
                    result: QueryIntrospectionResult::Unavailable,
                }
            };

            self.t.send_and_flush(reply).await.map_err(Into::into)
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    #[cfg(not(feature = "introspection"))]
    async fn msg_query_introspection(
        &mut self,
        msg: QueryIntrospection,
    ) -> Result<(), RunError<T::Error>> {
        self.t
            .send_and_flush(QueryIntrospectionReply {
                serial: msg.serial,
                result: QueryIntrospectionResult::Unavailable,
            })
            .await
            .map_err(Into::into)
    }

    #[cfg(feature = "introspection")]
    fn msg_query_introspection_reply(
        &mut self,
        msg: QueryIntrospectionReply,
    ) -> Result<(), RunError<T::Error>> {
        if self.protocol_version < ProtocolVersion::V1_17 {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        }

        let Some(req) = self.query_introspection.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        match msg.result {
            QueryIntrospectionResult::Ok(introspection) => {
                let _ = req
                    .reply
                    .send(Some(IntrospectionQueryResult::Serialized(introspection)));
            }

            QueryIntrospectionResult::Unavailable => {
                let _ = req.reply.send(None);
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "introspection"))]
    fn msg_query_introspection_reply(
        &mut self,
        msg: QueryIntrospectionReply,
    ) -> Result<(), RunError<T::Error>> {
        Err(RunError::UnexpectedMessageReceived(msg.into()))
    }

    fn msg_query_service_info_reply(
        &mut self,
        msg: QueryServiceInfoReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.query_service_info.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        debug_assert!(self.protocol_version >= ProtocolVersion::V1_17);

        let info = match msg.result {
            QueryServiceInfoResult::Ok(info) => {
                info.deserialize().map_err(RunError::Deserialize).map(Ok)?
            }

            QueryServiceInfoResult::InvalidService => Err(Error::InvalidService),
        };

        self.finish_create_proxy(req, info);
        Ok(())
    }

    fn msg_query_service_version_reply(
        &mut self,
        msg: QueryServiceVersionReply,
    ) -> Result<(), RunError<T::Error>> {
        let Some(req) = self.query_service_version.remove(msg.serial) else {
            return Err(RunError::UnexpectedMessageReceived(msg.into()));
        };

        // We never send QueryServiceVersion on protocol versions >= 1.17.
        debug_assert!(self.protocol_version < ProtocolVersion::V1_17);

        let info = match msg.result {
            QueryServiceVersionResult::Ok(version) => Ok(ServiceInfo::new(version)),
            QueryServiceVersionResult::InvalidService => Err(Error::InvalidService),
        };

        self.finish_create_proxy(req, info);
        Ok(())
    }

    fn finish_create_proxy(&mut self, req: CreateProxyRequest, info: Result<ServiceInfo, Error>) {
        let res = info.map(|info| self.proxies.create(self.handle.clone(), req.service, info));
        let _ = req.reply.send(res);
    }

    fn msg_subscribe_event_reply(
        &mut self,
        msg: SubscribeEventReply,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(req) = self.subscribe_event.remove(msg.serial) {
            let res = match msg.result {
                SubscribeEventResult::Ok => Ok(()),
                SubscribeEventResult::InvalidService => Err(Error::InvalidService),
            };

            let _ = req.reply.send(res);
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(msg.into()))
        }
    }

    fn msg_emit_event(&self, msg: EmitEvent) {
        self.proxies.emit(msg.service_cookie, msg.event, msg.value);
    }

    fn msg_service_destroyed(&mut self, msg: ServiceDestroyed) {
        self.proxies.remove_service(msg.service_cookie);
    }

    async fn handle_request(&mut self, req: HandleRequest) -> Result<(), RunError<T::Error>> {
        match req {
            HandleRequest::HandleCloned => self.req_handle_cloned(),
            HandleRequest::HandleDropped => self.req_handle_dropped(),
            HandleRequest::CreateObject(req) => self.req_create_object(req).await?,
            HandleRequest::DestroyObject(req) => self.req_destroy_object(req).await?,
            HandleRequest::CreateService(req) => self.req_create_service(req).await?,
            HandleRequest::DestroyService(req) => self.req_destroy_service(req).await?,
            HandleRequest::CallFunction(req) => self.req_call_function(req).await?,
            HandleRequest::CallFunctionReply(req) => self.req_call_function_reply(req).await?,
            HandleRequest::EmitEvent(req) => self.req_emit_event(req).await?,
            HandleRequest::CreateClaimedSender(req) => self.req_create_claimed_sender(req).await?,
            HandleRequest::CreateClaimedReceiver(req) => {
                self.req_create_claimed_receiver(req).await?
            }
            HandleRequest::CloseChannelEnd(req) => self.req_close_channel_end(req).await?,
            HandleRequest::ClaimSender(req) => self.req_claim_sender(req).await?,
            HandleRequest::ClaimReceiver(req) => self.req_claim_receiver(req).await?,
            HandleRequest::SendItem(req) => self.req_send_item(req).await?,
            HandleRequest::AddChannelCapacity(req) => self.req_add_channel_capacity(req).await?,
            HandleRequest::SyncClient(req) => self.req_sync_client(req),
            HandleRequest::SyncBroker(req) => self.req_sync_broker(req).await?,
            HandleRequest::CreateBusListener(req) => self.req_create_bus_listener(req).await?,
            HandleRequest::DestroyBusListener(req) => self.req_destroy_bus_listener(req).await?,
            HandleRequest::AddBusListenerFilter(req) => {
                self.req_add_bus_listener_filter(req).await?
            }
            HandleRequest::RemoveBusListenerFilter(req) => {
                self.req_remove_bus_listener_filter(req).await?
            }
            HandleRequest::ClearBusListenerFilters(req) => {
                self.req_clear_bus_listener_filters(req).await?
            }
            HandleRequest::StartBusListener(req) => self.req_start_bus_listener(req).await?,
            HandleRequest::StopBusListener(req) => self.req_stop_bus_listener(req).await?,
            HandleRequest::CreateLifetimeListener(req) => {
                self.req_create_lifetime_listener(req).await?
            }
            HandleRequest::GetProtocolVersion(req) => {
                let _ = req.send(self.protocol_version);
            }
            HandleRequest::CreateProxy(req) => self.req_create_proxy(req).await?,
            HandleRequest::DestroyProxy(proxy) => self.req_destroy_proxy(proxy).await?,
            HandleRequest::SubscribeEvent(req) => self.req_subscribe_event(req).await?,
            HandleRequest::UnsubscribeEvent(req) => self.req_unsubscribe_event(req).await?,
            #[cfg(feature = "introspection")]
            HandleRequest::RegisterIntrospection(introspection) => {
                self.introspection
                    .insert(introspection.type_id(), introspection);
            }
            #[cfg(feature = "introspection")]
            HandleRequest::SubmitIntrospection => self.req_submit_introspection().await?,
            #[cfg(feature = "introspection")]
            HandleRequest::QueryIntrospection(req) => self.req_query_introspection(req).await?,

            // Handled in Client::run()
            HandleRequest::Shutdown => unreachable!(),
        }

        Ok(())
    }

    fn req_handle_cloned(&mut self) {
        self.num_handles += 1;
    }

    fn req_handle_dropped(&mut self) {
        self.num_handles -= 1;
        debug_assert!(self.num_handles >= 1);
    }

    async fn req_create_object(
        &mut self,
        req: CreateObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let uuid = req.uuid;
        let serial = self.create_object.insert(req);

        self.t
            .send_and_flush(CreateObject { serial, uuid })
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_object(
        &mut self,
        req: DestroyObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.destroy_object.insert(req.reply);

        self.t
            .send_and_flush(DestroyObject {
                serial,
                cookie: req.cookie,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_create_service(
        &mut self,
        req: CreateServiceRequest,
    ) -> Result<(), RunError<T::Error>> {
        let object_cookie = req.object_id.cookie;
        let uuid = req.service_uuid;

        if self.protocol_version >= ProtocolVersion::V1_17 {
            let info = req.info;
            let serial = self.create_service.insert(req);

            let msg = CreateService2::with_serialize_info(serial, object_cookie, uuid, info)
                .map_err(RunError::Serialize)?;

            self.t.send_and_flush(msg).await.map_err(Into::into)
        } else {
            let version = req.info.version;
            let serial = self.create_service.insert(req);

            self.t
                .send_and_flush(CreateService {
                    serial,
                    object_cookie,
                    uuid,
                    version,
                })
                .await
                .map_err(Into::into)
        }
    }

    async fn req_destroy_service(
        &mut self,
        req: DestroyServiceRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.id.cookie;
        let serial = self.destroy_service.insert(req);

        self.t
            .send_and_flush(DestroyService { serial, cookie })
            .await
            .map_err(Into::into)
    }

    async fn req_call_function(
        &mut self,
        req: CallFunctionRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.function_calls.insert(req.reply);

        self.t
            .send_and_flush(CallFunction {
                serial,
                service_cookie: req.service_cookie,
                function: req.function,
                value: req.value,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_call_function_reply(
        &mut self,
        req: CallFunctionReplyRequest,
    ) -> Result<(), RunError<T::Error>> {
        self.abort_call_handles.remove(&req.serial);

        self.t
            .send_and_flush(CallFunctionReply {
                serial: req.serial,
                result: req.result,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_emit_event(&mut self, req: EmitEventRequest) -> Result<(), RunError<T::Error>> {
        let subscribed = self
            .broker_subscriptions
            .get(&req.service_cookie)
            .map(|events| events.contains(&req.event))
            .unwrap_or(false);

        if subscribed {
            self.t
                .send_and_flush(EmitEvent {
                    service_cookie: req.service_cookie,
                    event: req.event,
                    value: req.value,
                })
                .await?
        }

        Ok(())
    }

    async fn req_create_claimed_sender(
        &mut self,
        req: CreateClaimedSenderRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.create_channel.insert(CreateChannelData::Sender(req));

        self.t
            .send_and_flush(CreateChannel {
                serial,
                end: ChannelEndWithCapacity::Sender,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_create_claimed_receiver(
        &mut self,
        req: CreateClaimedReceiverRequest,
    ) -> Result<(), RunError<T::Error>> {
        let capacity = req.capacity.get();
        let serial = self.create_channel.insert(CreateChannelData::Receiver(req));

        self.t
            .send_and_flush(CreateChannel {
                serial,
                end: ChannelEndWithCapacity::Receiver(capacity),
            })
            .await
            .map_err(Into::into)
    }

    async fn req_close_channel_end(
        &mut self,
        req: CloseChannelEndRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;
        let end = req.end;

        let serial = self.close_channel_end.insert(req);

        self.t
            .send_and_flush(CloseChannelEnd {
                serial,
                cookie,
                end,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_claim_sender(
        &mut self,
        req: ClaimSenderRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;

        let serial = self
            .claim_channel_end
            .insert(ClaimChannelEndData::Sender(req));

        self.t
            .send_and_flush(ClaimChannelEnd {
                serial,
                cookie,
                end: ChannelEndWithCapacity::Sender,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_claim_receiver(
        &mut self,
        req: ClaimReceiverRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;
        let capacity = req.capacity.get();

        let serial = self
            .claim_channel_end
            .insert(ClaimChannelEndData::Receiver(req));

        self.t
            .send_and_flush(ClaimChannelEnd {
                serial,
                cookie,
                end: ChannelEndWithCapacity::Receiver(capacity),
            })
            .await
            .map_err(Into::into)
    }

    async fn req_send_item(&mut self, req: SendItemRequest) -> Result<(), RunError<T::Error>> {
        debug_assert!(self.senders.contains_key(&req.cookie));

        self.t
            .send_and_flush(SendItem {
                cookie: req.cookie,
                value: req.value,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_add_channel_capacity(
        &mut self,
        req: AddChannelCapacity,
    ) -> Result<(), RunError<T::Error>> {
        debug_assert!(self.receivers.contains_key(&req.cookie));
        self.t.send_and_flush(req).await.map_err(Into::into)
    }

    fn req_sync_client(&self, req: SyncClientRequest) {
        let _ = req.send(());
    }

    async fn req_sync_broker(&mut self, req: SyncBrokerRequest) -> Result<(), RunError<T::Error>> {
        let serial = self.sync.insert(req);

        self.t
            .send_and_flush(Sync { serial })
            .await
            .map_err(Into::into)
    }

    async fn req_create_bus_listener(
        &mut self,
        req: CreateBusListenerRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self
            .create_bus_listener
            .insert(CreateBusListenerData::BusListener(req));

        self.t
            .send_and_flush(CreateBusListener { serial })
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_bus_listener(
        &mut self,
        req: DestroyBusListenerRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;
        let serial = self.destroy_bus_listener.insert(req);

        self.t
            .send_and_flush(DestroyBusListener { serial, cookie })
            .await
            .map_err(Into::into)
    }

    async fn req_add_bus_listener_filter(
        &mut self,
        req: AddBusListenerFilter,
    ) -> Result<(), RunError<T::Error>> {
        let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
            return Ok(());
        };

        self.t.send_and_flush(req).await?;
        bus_listener.add_filter(req.filter);

        Ok(())
    }

    async fn req_remove_bus_listener_filter(
        &mut self,
        req: RemoveBusListenerFilter,
    ) -> Result<(), RunError<T::Error>> {
        let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
            return Ok(());
        };

        self.t.send_and_flush(req).await?;
        bus_listener.remove_filter(req.filter);

        Ok(())
    }

    async fn req_clear_bus_listener_filters(
        &mut self,
        req: ClearBusListenerFilters,
    ) -> Result<(), RunError<T::Error>> {
        let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
            return Ok(());
        };

        self.t.send_and_flush(req).await?;
        bus_listener.clear_filters();

        Ok(())
    }

    async fn req_start_bus_listener(
        &mut self,
        req: StartBusListenerRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;
        let scope = req.scope;
        let serial = self.start_bus_listener.insert(req);

        self.t
            .send_and_flush(StartBusListener {
                serial,
                cookie,
                scope,
            })
            .await
            .map_err(Into::into)
    }

    async fn req_stop_bus_listener(
        &mut self,
        req: StopBusListenerRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.cookie;
        let serial = self.stop_bus_listener.insert(req);

        self.t
            .send_and_flush(StopBusListener { serial, cookie })
            .await
            .map_err(Into::into)
    }

    async fn req_create_lifetime_listener(
        &mut self,
        req: CreateLifetimeListenerRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self
            .create_bus_listener
            .insert(CreateBusListenerData::LifetimeListener(req));

        self.t
            .send_and_flush(CreateBusListener { serial })
            .await
            .map_err(Into::into)
    }

    async fn req_create_proxy(
        &mut self,
        req: CreateProxyRequest,
    ) -> Result<(), RunError<T::Error>> {
        let msg = if self.protocol_version >= ProtocolVersion::V1_17 {
            let cookie = req.service.cookie;
            let serial = self.query_service_info.insert(req);
            Message::QueryServiceInfo(QueryServiceInfo { serial, cookie })
        } else {
            let cookie = req.service.cookie;
            let serial = self.query_service_version.insert(req);
            Message::QueryServiceVersion(QueryServiceVersion { serial, cookie })
        };

        self.t.send_and_flush(msg).await.map_err(Into::into)
    }

    async fn req_destroy_proxy(&mut self, proxy: ProxyId) -> Result<(), RunError<T::Error>> {
        if let Some((service_cookie, events)) = self.proxies.remove(proxy) {
            if !events.is_empty() {
                for event in events {
                    self.t
                        .send(UnsubscribeEvent {
                            service_cookie,
                            event,
                        })
                        .await?;
                }

                self.t.flush().await?;
            }
        }

        Ok(())
    }

    async fn req_subscribe_event(
        &mut self,
        req: SubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        match self.proxies.subscribe(req.proxy, req.event) {
            SubscribeResult::Forward(service_cookie) => {
                let event = req.event;
                let serial = self.subscribe_event.insert(req);

                self.t
                    .send_and_flush(SubscribeEvent {
                        serial: Some(serial),
                        service_cookie,
                        event,
                    })
                    .await?;
            }

            SubscribeResult::Noop => {
                let _ = req.reply.send(Ok(()));
            }

            SubscribeResult::InvalidProxy => {
                let _ = req.reply.send(Err(Error::InvalidService));
            }
        }

        Ok(())
    }

    async fn req_unsubscribe_event(
        &mut self,
        req: UnsubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        match self.proxies.unsubscribe(req.proxy, req.event) {
            SubscribeResult::Forward(service_cookie) => {
                self.t
                    .send_and_flush(UnsubscribeEvent {
                        service_cookie,
                        event: req.event,
                    })
                    .await?;

                let _ = req.reply.send(Ok(()));
            }

            SubscribeResult::Noop => {
                let _ = req.reply.send(Ok(()));
            }

            SubscribeResult::InvalidProxy => {
                let _ = req.reply.send(Err(Error::InvalidService));
            }
        }

        Ok(())
    }

    #[cfg(feature = "introspection")]
    async fn req_submit_introspection(&mut self) -> Result<(), RunError<T::Error>> {
        use crate::core::message::RegisterIntrospection;

        if (self.protocol_version >= ProtocolVersion::V1_17) && !self.introspection.is_empty() {
            let type_ids = self.introspection.keys().copied().collect();

            let register_introspection = RegisterIntrospection::with_serialize_type_ids(&type_ids)
                .map_err(RunError::Serialize)?;

            self.t
                .send_and_flush(register_introspection)
                .await
                .map_err(Into::into)
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "introspection")]
    async fn req_query_introspection(
        &mut self,
        req: QueryIntrospectionRequest,
    ) -> Result<(), RunError<T::Error>> {
        if let Some(introspection) = self.introspection.get(&req.type_id) {
            let _ = req
                .reply
                .send(Some(IntrospectionQueryResult::Local(introspection)));
            Ok(())
        } else if self.protocol_version >= ProtocolVersion::V1_17 {
            let type_id = req.type_id;
            let serial = self.query_introspection.insert(req);

            self.t
                .send_and_flush(QueryIntrospection { serial, type_id })
                .await
                .map_err(Into::into)
        } else {
            let _ = req.reply.send(None);
            Ok(())
        }
    }

    async fn abort_function_call(&mut self, serial: u32) -> Result<(), RunError<T::Error>> {
        self.function_calls.abort(serial);

        if self.protocol_version >= ProtocolVersion::V1_16 {
            self.t.send_and_flush(AbortFunctionCall { serial }).await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum CreateChannelData {
    Sender(CreateClaimedSenderRequest),
    Receiver(CreateClaimedReceiverRequest),
}

#[derive(Debug)]
enum ClaimChannelEndData {
    Sender(ClaimSenderRequest),
    Receiver(ClaimReceiverRequest),
}

#[derive(Debug)]
enum SenderState {
    Pending(oneshot::Sender<(u32, mpsc::UnboundedReceiver<u32>)>),
    Established(mpsc::UnboundedSender<u32>),
    ReceiverClosed,
}

#[derive(Debug)]
enum ReceiverState {
    Pending(oneshot::Sender<mpsc::UnboundedReceiver<SerializedValue>>),
    Established(mpsc::UnboundedSender<SerializedValue>),
    SenderClosed,
}

#[derive(Debug)]
enum CreateBusListenerData {
    BusListener(CreateBusListenerRequest),
    LifetimeListener(CreateLifetimeListenerRequest),
}
