use crate::channel::{
    PendingReceiverInner, PendingSenderInner, ReceiverInner, SenderInner, UnclaimedReceiverInner,
    UnclaimedSenderInner,
};
use crate::error::{ConnectError, RunError};
use crate::events::{EventsId, EventsRequest};
use crate::handle::request::{
    CallFunctionReplyRequest, CallFunctionRequest, ClaimReceiverRequest, ClaimSenderRequest,
    CloseChannelEndRequest, CreateClaimedReceiverRequest, CreateClaimedSenderRequest,
    CreateObjectRequest, CreateServiceRequest, DestroyObjectRequest, DestroyServiceRequest,
    EmitEventRequest, HandleRequest, QueryServiceVersionRequest, SendItemRequest,
    SubscribeEventRequest, SyncBrokerRequest, SyncClientRequest, UnsubscribeEventRequest,
};
use crate::serial_map::SerialMap;
use crate::service::RawFunctionCall;
use crate::{Error, Handle, Object, Service};
use aldrin_proto::message::{
    AddChannelCapacity, CallFunction, CallFunctionReply, CallFunctionResult, ChannelEnd,
    ChannelEndClaimed, ChannelEndClosed, ChannelEndWithCapacity, ClaimChannelEnd,
    ClaimChannelEndReply, ClaimChannelEndResult, CloseChannelEnd, CloseChannelEndReply,
    CloseChannelEndResult, Connect, ConnectReply, CreateChannel, CreateChannelReply, CreateObject,
    CreateObjectReply, CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult,
    DestroyObject, DestroyObjectReply, DestroyObjectResult, DestroyService, DestroyServiceReply,
    DestroyServiceResult, EmitEvent, ItemReceived, Message, QueryServiceVersion,
    QueryServiceVersionReply, QueryServiceVersionResult, SendItem, ServiceDestroyed, Shutdown,
    SubscribeEvent, SubscribeEventReply, SubscribeEventResult, Sync, SyncReply, UnsubscribeEvent,
};
use aldrin_proto::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_proto::{
    ChannelCookie, Deserialize, ObjectId, Serialize, SerializedValue, ServiceCookie, ServiceId,
};
use futures_channel::{mpsc, oneshot};
use futures_util::future::{select, Either};
use futures_util::stream::StreamExt;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::mem;

type Subscriptions = HashMap<u32, HashMap<EventsId, mpsc::UnboundedSender<EventsRequest>>>;

/// Aldrin client used to connect to a broker.
///
/// This is the first entry point to `aldrin-client`. A [`Client`] is used to establish a connection
/// to an Aldrin broker. Afterwards, it should be turned into a [`Future`](std::future::Future) with
/// the [`run`](Client::run) method, which must then be continuously polled and run to completion.
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
    t: T,
    recv: mpsc::UnboundedReceiver<HandleRequest>,
    handle: Handle,
    num_handles: usize,
    create_object: SerialMap<CreateObjectRequest>,
    destroy_object: SerialMap<oneshot::Sender<DestroyObjectResult>>,
    create_service: SerialMap<CreateServiceRequest>,
    destroy_service: SerialMap<DestroyServiceRequest>,
    function_calls: SerialMap<oneshot::Sender<CallFunctionResult>>,
    services: HashMap<ServiceCookie, mpsc::UnboundedSender<RawFunctionCall>>,
    subscribe_event: SerialMap<(
        EventsId,
        ServiceCookie,
        u32,
        oneshot::Sender<SubscribeEventResult>,
    )>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
    broker_subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
    query_service_version: SerialMap<oneshot::Sender<QueryServiceVersionResult>>,
    create_channel: SerialMap<CreateChannelData>,
    close_channel_end: SerialMap<CloseChannelEndRequest>,
    claim_channel_end: SerialMap<ClaimChannelEndData>,
    senders: HashMap<ChannelCookie, SenderState>,
    receivers: HashMap<ChannelCookie, ReceiverState>,
    sync: SerialMap<SyncBrokerRequest>,
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
    /// use aldrin_client::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_test::tokio_based::TestBroker::new();
    /// # let mut handle = broker.clone();
    /// # let (async_transport, t2) = aldrin_channel::unbounded();
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
        let (client, _) = Self::connect_with_data(t, &()).await?;
        Ok(client)
    }

    /// Creates a client and connects to an Aldrin broker. Allows to send and receive custom data.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
    pub async fn connect_with_data<D: Serialize + ?Sized>(
        mut t: T,
        data: &D,
    ) -> Result<(Self, SerializedValue), ConnectError<T::Error>> {
        let connect = Connect::with_serialize_value(aldrin_proto::VERSION, data)?;
        t.send_and_flush(Message::Connect(connect))
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = match t.receive().await.map_err(ConnectError::Transport)? {
            Message::ConnectReply(connect_reply) => connect_reply,
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg)),
        };

        let data = match connect_reply {
            ConnectReply::Ok(data) => data,
            ConnectReply::VersionMismatch(v) => return Err(ConnectError::VersionMismatch(v)),
            ConnectReply::Rejected(data) => return Err(ConnectError::Rejected(data)),
        };

        let (send, recv) = mpsc::unbounded();
        let client = Client {
            t,
            recv,
            handle: Handle::new(send),
            num_handles: 1,
            create_object: SerialMap::new(),
            destroy_object: SerialMap::new(),
            create_service: SerialMap::new(),
            destroy_service: SerialMap::new(),
            function_calls: SerialMap::new(),
            services: HashMap::new(),
            subscribe_event: SerialMap::new(),
            subscriptions: HashMap::new(),
            broker_subscriptions: HashMap::new(),
            query_service_version: SerialMap::new(),
            create_channel: SerialMap::new(),
            close_channel_end: SerialMap::new(),
            claim_channel_end: SerialMap::new(),
            senders: HashMap::new(),
            receivers: HashMap::new(),
            sync: SerialMap::new(),
        };

        Ok((client, data))
    }

    /// Creates a client and connects to an Aldrin broker. Allows to send and receive custom data.
    ///
    /// After creating a client, it must be continuously polled and run to completion with the
    /// [`run`](Client::run) method.
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
    /// # tokio::spawn(async move { handle.connect(t2).await });
    /// // Create an AsyncTransport for connecting to the broker.
    /// // let async_transport = ...
    ///
    /// // Connect to the broker, sending some custom data.
    /// let (client, data) = Client::connect_with_data(async_transport, "Hi!").await?;
    ///
    /// println!("Data the broker sent back: {:?}.", data);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_data_and_deserialize<D1, D2>(
        t: T,
        data: &D1,
    ) -> Result<(Self, D2), ConnectError<T::Error>>
    where
        D1: Serialize + ?Sized,
        D2: Deserialize,
    {
        let (client, data) = Self::connect_with_data(t, data).await?;
        let data = data.deserialize()?;
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
            match select(self.t.receive(), self.recv.next()).await {
                Either::Left((Ok(Message::Shutdown(Shutdown)), _)) => {
                    self.t.send_and_flush(Message::Shutdown(Shutdown)).await?;
                    return Ok(());
                }

                Either::Left((Ok(msg), _)) => self.handle_message(msg).await?,
                Either::Left((Err(e), _)) => return Err(e.into()),
                Either::Right((Some(HandleRequest::Shutdown), _)) => break,
                Either::Right((Some(req), _)) => self.handle_request(req).await?,

                // Unreachable, because Client holds a sender.
                Either::Right((None, _)) => unreachable!(),
            }

            if self.num_handles == 1 {
                break;
            }
        }

        self.t.send_and_flush(Message::Shutdown(Shutdown)).await?;
        self.drain_transport().await?;
        Ok(())
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
            Message::SubscribeEventReply(msg) => self.msg_subscribe_event_reply(msg),
            Message::UnsubscribeEvent(msg) => self.msg_unsubscribe_event(msg),
            Message::EmitEvent(msg) => self.msg_emit_event(msg),
            Message::QueryServiceVersionReply(msg) => self.msg_query_service_version_reply(msg),
            Message::CreateChannelReply(msg) => self.msg_create_channel_reply(msg)?,
            Message::CloseChannelEndReply(msg) => self.msg_close_channel_end_reply(msg)?,
            Message::ChannelEndClosed(msg) => self.msg_channel_end_closed(msg)?,
            Message::ClaimChannelEndReply(msg) => self.msg_claim_channel_end_reply(msg)?,
            Message::ChannelEndClaimed(msg) => self.msg_channel_end_claimed(msg)?,
            Message::ItemReceived(msg) => self.msg_item_received(msg)?,
            Message::AddChannelCapacity(msg) => self.msg_add_channel_capacity(msg)?,
            Message::SyncReply(msg) => self.msg_sync_reply(msg)?,
            Message::ServiceDestroyed(msg) => self.msg_service_destroyed(msg),
            Message::CreateBusListenerReply(_) => todo!(),
            Message::DestroyBusListenerReply(_) => todo!(),

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
            | Message::StartBusListener(_) => return Err(RunError::UnexpectedMessageReceived(msg)),

            Message::Shutdown(Shutdown) => unreachable!(), // Handled in run.
        }

        Ok(())
    }

    fn msg_create_object_reply(
        &mut self,
        msg: CreateObjectReply,
    ) -> Result<(), RunError<T::Error>> {
        let req = match self.create_object.remove(msg.serial) {
            Some(req) => req,
            None => {
                return Err(RunError::UnexpectedMessageReceived(
                    Message::CreateObjectReply(msg),
                ))
            }
        };

        let reply = match msg.result {
            CreateObjectResult::Ok(cookie) => Ok(Object::new(
                ObjectId::new(req.uuid, cookie),
                self.handle.clone(),
            )),
            CreateObjectResult::DuplicateObject => Err(Error::DuplicateObject(req.uuid)),
        };

        req.reply.send(reply).ok();
        Ok(())
    }

    fn msg_destroy_object_reply(&mut self, msg: DestroyObjectReply) {
        if let Some(send) = self.destroy_object.remove(msg.serial) {
            send.send(msg.result).ok();
        }
    }

    fn msg_create_service_reply(
        &mut self,
        msg: CreateServiceReply,
    ) -> Result<(), RunError<T::Error>> {
        let req = match self.create_service.remove(msg.serial) {
            Some(req) => req,
            None => {
                return Err(RunError::UnexpectedMessageReceived(
                    Message::CreateServiceReply(msg),
                ))
            }
        };

        let reply = match msg.result {
            CreateServiceResult::Ok(cookie) => {
                let (send, function_calls) = mpsc::unbounded();
                let dup = self.services.insert(cookie, send);
                debug_assert!(dup.is_none());
                Ok(Service::new(
                    ServiceId::new(req.object_id, req.service_uuid, cookie),
                    req.version,
                    self.handle.clone(),
                    function_calls,
                ))
            }
            CreateServiceResult::DuplicateService => {
                Err(Error::DuplicateService(req.object_id, req.service_uuid))
            }
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject(req.object_id)),
            CreateServiceResult::ForeignObject => unreachable!(),
        };

        req.reply.send(reply).ok();
        Ok(())
    }

    fn msg_destroy_service_reply(&mut self, msg: DestroyServiceReply) {
        let req = match self.destroy_service.remove(msg.serial) {
            Some(req) => req,
            None => return,
        };

        let reply = match msg.result {
            DestroyServiceResult::Ok => {
                let contained = self.services.remove(&req.id.cookie);
                debug_assert!(contained.is_some());
                self.broker_subscriptions.remove(&req.id.cookie);
                Ok(())
            }
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(req.id)),
            DestroyServiceResult::ForeignObject => unreachable!(),
        };

        req.reply.send(reply).ok();
    }

    async fn msg_call_function(&mut self, msg: CallFunction) -> Result<(), RunError<T::Error>> {
        let send = self
            .services
            .get_mut(&msg.service_cookie)
            .expect("inconsistent state");
        let req = RawFunctionCall {
            serial: msg.serial,
            function: msg.function,
            value: msg.value,
        };

        if send.unbounded_send(req).is_err() {
            self.t
                .send_and_flush(Message::CallFunctionReply(CallFunctionReply {
                    serial: msg.serial,
                    result: CallFunctionResult::InvalidService,
                }))
                .await?;
        }

        Ok(())
    }

    fn msg_call_function_reply(&mut self, msg: CallFunctionReply) {
        let send = self
            .function_calls
            .remove(msg.serial)
            .expect("inconsistent state");
        send.send(msg.result).ok();
    }

    fn msg_subscribe_event(&mut self, msg: SubscribeEvent) {
        self.broker_subscriptions
            .entry(msg.service_cookie)
            .or_default()
            .insert(msg.event);
    }

    fn msg_subscribe_event_reply(&mut self, msg: SubscribeEventReply) {
        let (events_id, service_cookie, id, rep_send) =
            match self.subscribe_event.remove(msg.serial) {
                Some(req) => req,
                None => return,
            };

        // || would short-circuit and changing the order would move msg.result out.
        let mut err = msg.result != SubscribeEventResult::Ok;
        err |= rep_send.send(msg.result).is_err();
        if err {
            self.subscriptions
                .entry(service_cookie)
                .or_default()
                .entry(id)
                .or_default()
                .remove(&events_id);
        }
    }

    fn msg_unsubscribe_event(&mut self, msg: UnsubscribeEvent) {
        let mut subs = match self.broker_subscriptions.entry(msg.service_cookie) {
            Entry::Occupied(subs) => subs,
            Entry::Vacant(_) => return,
        };

        subs.get_mut().remove(&msg.event);
        if subs.get().is_empty() {
            subs.remove();
        }
    }

    fn msg_emit_event(&mut self, msg: EmitEvent) {
        let senders = match self
            .subscriptions
            .get_mut(&msg.service_cookie)
            .and_then(|s| s.get_mut(&msg.event))
        {
            Some(senders) => senders,
            None => return,
        };

        for sender in senders.values_mut() {
            // Should we close the channel in case of send errors?
            sender
                .unbounded_send(EventsRequest::EmitEvent(
                    msg.service_cookie,
                    msg.event,
                    msg.value.clone(),
                ))
                .ok();
        }
    }

    fn msg_query_service_version_reply(&mut self, msg: QueryServiceVersionReply) {
        if let Some(send) = self.query_service_version.remove(msg.serial) {
            send.send(msg.result).ok();
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
                reply.send((sender, receiver)).ok();
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
                req.reply.send((sender, receiver)).ok();
                Ok(())
            }

            None => Err(RunError::UnexpectedMessageReceived(
                Message::CreateChannelReply(msg),
            )),
        }
    }

    fn msg_close_channel_end_reply(
        &mut self,
        msg: CloseChannelEndReply,
    ) -> Result<(), RunError<T::Error>> {
        let req = match self.close_channel_end.remove(msg.serial) {
            Some(req) => req,
            None => {
                return Err(RunError::UnexpectedMessageReceived(
                    Message::CloseChannelEndReply(msg),
                ))
            }
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
            CloseChannelEndResult::InvalidChannel => Err(Error::InvalidChannel),
            CloseChannelEndResult::ForeignChannel => Err(Error::ForeignChannel),
        };

        req.reply.send(res).ok();
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
                    Some(ReceiverState::SenderClosed) | None => Err(
                        RunError::UnexpectedMessageReceived(Message::ChannelEndClosed(msg)),
                    ),
                }
            }

            ChannelEnd::Receiver => {
                let sender = self
                    .senders
                    .get_mut(&msg.cookie)
                    .map(|sender| mem::replace(sender, SenderState::ReceiverClosed));

                match sender {
                    Some(SenderState::Pending(_)) | Some(SenderState::Established(_)) => Ok(()),
                    Some(SenderState::ReceiverClosed) | None => Err(
                        RunError::UnexpectedMessageReceived(Message::ChannelEndClosed(msg)),
                    ),
                }
            }
        }
    }

    fn msg_claim_channel_end_reply(
        &mut self,
        msg: ClaimChannelEndReply,
    ) -> Result<(), RunError<T::Error>> {
        let req = match self.claim_channel_end.remove(msg.serial) {
            Some(req) => req,
            None => {
                return Err(RunError::UnexpectedMessageReceived(
                    Message::ClaimChannelEndReply(msg),
                ))
            }
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
                    req.reply.send(Ok(sender)).ok();
                }

                ClaimChannelEndResult::ReceiverClaimed => {
                    return Err(RunError::UnexpectedMessageReceived(
                        Message::ClaimChannelEndReply(msg),
                    ))
                }

                ClaimChannelEndResult::InvalidChannel => {
                    req.reply.send(Err(Error::InvalidChannel)).ok();
                }

                ClaimChannelEndResult::AlreadyClaimed => {
                    req.reply.send(Err(Error::ForeignChannel)).ok();
                }
            },

            ClaimChannelEndData::Receiver(req) => match msg.result {
                ClaimChannelEndResult::SenderClaimed(_) => {
                    return Err(RunError::UnexpectedMessageReceived(
                        Message::ClaimChannelEndReply(msg),
                    ))
                }

                ClaimChannelEndResult::ReceiverClaimed => {
                    let (send, recv) = mpsc::unbounded();
                    let dup = self
                        .receivers
                        .insert(req.cookie, ReceiverState::Established(send));
                    debug_assert!(dup.is_none());
                    let receiver =
                        ReceiverInner::new(req.cookie, self.handle.clone(), recv, req.capacity);
                    req.reply.send(Ok(receiver)).ok();
                }

                ClaimChannelEndResult::InvalidChannel => {
                    req.reply.send(Err(Error::InvalidChannel)).ok();
                }

                ClaimChannelEndResult::AlreadyClaimed => {
                    req.reply.send(Err(Error::ForeignChannel)).ok();
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
                let receiver = match self.receivers.get_mut(&msg.cookie) {
                    Some(receiver) => receiver,
                    None => {
                        return Err(RunError::UnexpectedMessageReceived(
                            Message::ChannelEndClaimed(msg),
                        ))
                    }
                };

                let (send, recv) = mpsc::unbounded();

                match mem::replace(receiver, ReceiverState::Established(send)) {
                    ReceiverState::Pending(send) => {
                        send.send(recv).ok();
                        Ok(())
                    }

                    ReceiverState::Established(_) | ReceiverState::SenderClosed => Err(
                        RunError::UnexpectedMessageReceived(Message::ChannelEndClaimed(msg)),
                    ),
                }
            }

            ChannelEndWithCapacity::Receiver(capacity) => {
                let sender = match self.senders.get_mut(&msg.cookie) {
                    Some(sender) => sender,
                    None => {
                        return Err(RunError::UnexpectedMessageReceived(
                            Message::ChannelEndClaimed(msg),
                        ))
                    }
                };

                let (send, recv) = mpsc::unbounded();

                match mem::replace(sender, SenderState::Established(send)) {
                    SenderState::Pending(send) => {
                        send.send((capacity, recv)).ok();
                        Ok(())
                    }

                    SenderState::Established(_) | SenderState::ReceiverClosed => Err(
                        RunError::UnexpectedMessageReceived(Message::ChannelEndClaimed(msg)),
                    ),
                }
            }
        }
    }

    fn msg_item_received(&self, msg: ItemReceived) -> Result<(), RunError<T::Error>> {
        if let Some(ReceiverState::Established(send)) = self.receivers.get(&msg.cookie) {
            send.unbounded_send(msg.value).ok();
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(Message::ItemReceived(
                msg,
            )))
        }
    }

    fn msg_add_channel_capacity(&self, msg: AddChannelCapacity) -> Result<(), RunError<T::Error>> {
        if let Some(SenderState::Established(send)) = self.senders.get(&msg.cookie) {
            send.unbounded_send(msg.capacity).ok();
            Ok(())
        } else {
            Err(RunError::UnexpectedMessageReceived(
                Message::AddChannelCapacity(msg),
            ))
        }
    }

    fn msg_sync_reply(&mut self, msg: SyncReply) -> Result<(), RunError<T::Error>> {
        let req = match self.sync.remove(msg.serial) {
            Some(req) => req,
            None => return Err(RunError::UnexpectedMessageReceived(Message::SyncReply(msg))),
        };

        req.send(()).ok();
        Ok(())
    }

    fn msg_service_destroyed(&mut self, msg: ServiceDestroyed) {
        let Some(ids) = self.subscriptions.remove(&msg.service_cookie) else {
            return;
        };

        let mut dups = HashSet::new();
        for (_, events_ids) in ids {
            for (events_id, sender) in events_ids {
                if dups.insert(events_id) {
                    // Should we close the channel in case of send errors?
                    sender
                        .unbounded_send(EventsRequest::ServiceDestroyed(msg.service_cookie))
                        .ok();
                }
            }
        }
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
            HandleRequest::SubscribeEvent(req) => self.req_subscribe_event(req).await?,
            HandleRequest::UnsubscribeEvent(req) => self.req_unsubscribe_event(req).await?,
            HandleRequest::EmitEvent(req) => self.req_emit_event(req).await?,
            HandleRequest::QueryServiceVersion(req) => self.req_query_service_version(req).await?,
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
            .send_and_flush(Message::CreateObject(CreateObject { serial, uuid }))
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_object(
        &mut self,
        req: DestroyObjectRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.destroy_object.insert(req.reply);
        self.t
            .send_and_flush(Message::DestroyObject(DestroyObject {
                serial,
                cookie: req.cookie,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_create_service(
        &mut self,
        req: CreateServiceRequest,
    ) -> Result<(), RunError<T::Error>> {
        let object_cookie = req.object_id.cookie;
        let uuid = req.service_uuid;
        let version = req.version;
        let serial = self.create_service.insert(req);
        self.t
            .send_and_flush(Message::CreateService(CreateService {
                serial,
                object_cookie,
                uuid,
                version,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_destroy_service(
        &mut self,
        req: DestroyServiceRequest,
    ) -> Result<(), RunError<T::Error>> {
        let cookie = req.id.cookie;
        let serial = self.destroy_service.insert(req);
        self.t
            .send_and_flush(Message::DestroyService(DestroyService { serial, cookie }))
            .await
            .map_err(Into::into)
    }

    async fn req_call_function(
        &mut self,
        req: CallFunctionRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.function_calls.insert(req.reply);
        self.t
            .send_and_flush(Message::CallFunction(CallFunction {
                serial,
                service_cookie: req.service_cookie,
                function: req.function,
                value: req.value,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_call_function_reply(
        &mut self,
        req: CallFunctionReplyRequest,
    ) -> Result<(), RunError<T::Error>> {
        self.t
            .send_and_flush(Message::CallFunctionReply(CallFunctionReply {
                serial: req.serial,
                result: req.result,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_subscribe_event(
        &mut self,
        req: SubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        let subs = self
            .subscriptions
            .entry(req.service_cookie)
            .or_default()
            .entry(req.id)
            .or_default();
        let send_req = subs.is_empty();
        let duplicate = subs.insert(req.events_id, req.sender).is_some();

        if send_req {
            let serial = Some(self.subscribe_event.insert((
                req.events_id,
                req.service_cookie,
                req.id,
                req.reply,
            )));

            self.t
                .send_and_flush(Message::SubscribeEvent(SubscribeEvent {
                    serial,
                    service_cookie: req.service_cookie,
                    event: req.id,
                }))
                .await
                .map_err(Into::into)
        } else {
            if req.reply.send(SubscribeEventResult::Ok).is_err() && !duplicate {
                subs.remove(&req.events_id);
            }

            Ok(())
        }
    }

    async fn req_unsubscribe_event(
        &mut self,
        req: UnsubscribeEventRequest,
    ) -> Result<(), RunError<T::Error>> {
        let mut subs = match self.subscriptions.entry(req.service_cookie) {
            Entry::Occupied(subs) => subs,
            Entry::Vacant(_) => return Ok(()),
        };

        let mut ids = match subs.get_mut().entry(req.id) {
            Entry::Occupied(ids) => ids,
            Entry::Vacant(_) => return Ok(()),
        };

        if ids.get_mut().remove(&req.events_id).is_none() {
            return Ok(());
        }

        if !ids.get().is_empty() {
            return Ok(());
        }

        ids.remove();
        if subs.get().is_empty() {
            subs.remove();
        }

        self.t
            .send_and_flush(Message::UnsubscribeEvent(UnsubscribeEvent {
                service_cookie: req.service_cookie,
                event: req.id,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_emit_event(&mut self, req: EmitEventRequest) -> Result<(), RunError<T::Error>> {
        self.t
            .send_and_flush(Message::EmitEvent(EmitEvent {
                service_cookie: req.service_cookie,
                event: req.event,
                value: req.value,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_query_service_version(
        &mut self,
        req: QueryServiceVersionRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.query_service_version.insert(req.reply);
        self.t
            .send_and_flush(Message::QueryServiceVersion(QueryServiceVersion {
                serial,
                cookie: req.cookie,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_create_claimed_sender(
        &mut self,
        req: CreateClaimedSenderRequest,
    ) -> Result<(), RunError<T::Error>> {
        let serial = self.create_channel.insert(CreateChannelData::Sender(req));
        self.t
            .send_and_flush(Message::CreateChannel(CreateChannel {
                serial,
                end: ChannelEndWithCapacity::Sender,
            }))
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
            .send_and_flush(Message::CreateChannel(CreateChannel {
                serial,
                end: ChannelEndWithCapacity::Receiver(capacity),
            }))
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
            .send_and_flush(Message::CloseChannelEnd(CloseChannelEnd {
                serial,
                cookie,
                end,
            }))
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
            .send_and_flush(Message::ClaimChannelEnd(ClaimChannelEnd {
                serial,
                cookie,
                end: ChannelEndWithCapacity::Sender,
            }))
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
            .send_and_flush(Message::ClaimChannelEnd(ClaimChannelEnd {
                serial,
                cookie,
                end: ChannelEndWithCapacity::Receiver(capacity),
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_send_item(&mut self, req: SendItemRequest) -> Result<(), RunError<T::Error>> {
        debug_assert!(self.senders.contains_key(&req.cookie));

        self.t
            .send_and_flush(Message::SendItem(SendItem {
                cookie: req.cookie,
                value: req.value,
            }))
            .await
            .map_err(Into::into)
    }

    async fn req_add_channel_capacity(
        &mut self,
        req: AddChannelCapacity,
    ) -> Result<(), RunError<T::Error>> {
        debug_assert!(self.receivers.contains_key(&req.cookie));

        self.t
            .send_and_flush(Message::AddChannelCapacity(req))
            .await
            .map_err(Into::into)
    }

    fn req_sync_client(&self, req: SyncClientRequest) {
        req.send(()).ok();
    }

    async fn req_sync_broker(&mut self, req: SyncBrokerRequest) -> Result<(), RunError<T::Error>> {
        let serial = self.sync.insert(req);
        self.t
            .send_and_flush(Message::Sync(Sync { serial }))
            .await
            .map_err(Into::into)
    }

    fn shutdown_all_events(&self) {
        for by_service in self.subscriptions.values() {
            for by_function in by_service.values() {
                for events in by_function.values() {
                    events.close_channel();
                }
            }
        }
    }
}

impl<T> Drop for Client<T>
where
    T: AsyncTransport + Unpin,
{
    fn drop(&mut self) {
        self.shutdown_all_events();
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
