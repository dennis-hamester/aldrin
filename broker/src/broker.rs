mod channel;
mod conn_state;
mod error;
mod handle;
mod object;
mod service;
mod state;
#[cfg(feature = "statistics")]
mod statistics;
#[cfg(test)]
mod test;

use crate::bus_listener::BusListener;
use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionId;
use crate::core::message::{
    AbortFunctionCall, AddBusListenerFilter, AddChannelCapacity, BusListenerCurrentFinished,
    CallFunction, CallFunctionReply, CallFunctionResult, ChannelEndClaimed, ChannelEndClosed,
    ClaimChannelEnd, ClaimChannelEndReply, ClaimChannelEndResult, ClearBusListenerFilters,
    CloseChannelEnd, CloseChannelEndReply, CloseChannelEndResult, CreateBusListener,
    CreateBusListenerReply, CreateChannel, CreateChannelReply, CreateObject, CreateObjectReply,
    CreateObjectResult, CreateService, CreateService2, CreateServiceReply, CreateServiceResult,
    DestroyBusListener, DestroyBusListenerReply, DestroyBusListenerResult, DestroyObject,
    DestroyObjectReply, DestroyObjectResult, DestroyService, DestroyServiceReply,
    DestroyServiceResult, EmitBusEvent, EmitEvent, ItemReceived, Message, QueryIntrospection,
    QueryIntrospectionReply, QueryIntrospectionResult, QueryServiceInfo, QueryServiceInfoReply,
    QueryServiceInfoResult, QueryServiceVersion, QueryServiceVersionReply,
    QueryServiceVersionResult, RegisterIntrospection, RemoveBusListenerFilter, SendItem,
    ServiceDestroyed, Shutdown, StartBusListener, StartBusListenerReply, StartBusListenerResult,
    StopBusListener, StopBusListenerReply, StopBusListenerResult, SubscribeAllEvents,
    SubscribeAllEventsReply, SubscribeAllEventsResult, SubscribeEvent, SubscribeEventReply,
    SubscribeEventResult, SubscribeService, SubscribeServiceReply, SubscribeServiceResult, Sync,
    SyncReply, UnsubscribeAllEvents, UnsubscribeAllEventsReply, UnsubscribeAllEventsResult,
    UnsubscribeEvent, UnsubscribeService,
};
#[cfg(feature = "introspection")]
use crate::core::TypeId;
use crate::core::{
    BusEvent, BusListenerCookie, BusListenerScope, ChannelCookie, ChannelEnd,
    ChannelEndWithCapacity, ObjectCookie, ObjectId, ObjectUuid, ProtocolVersion, ServiceCookie,
    ServiceId, ServiceInfo, ServiceUuid,
};
#[cfg(feature = "introspection")]
use crate::introspection_database::{
    IntrospectionDatabase, IntrospectionQueryResult, RemoveConnResult,
};
use crate::serial_map::SerialMap;
use channel::{AddCapacityError, Channel, SendItemError};
use conn_state::ConnectionState;
use futures_channel::mpsc::{channel, Receiver};
use futures_util::stream::StreamExt;
use object::Object;
use service::Service;
use state::State;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

pub use error::BrokerShutdown;
pub use handle::{BrokerHandle, PendingConnection};
#[cfg(feature = "statistics")]
pub use statistics::BrokerStatistics;

const FIFO_SIZE: usize = 32;

macro_rules! send {
    ($self:expr, $conn:expr, $msg:expr $(,)?) => {{
        let res = $conn.send($msg.into());

        #[cfg(feature = "statistics")]
        {
            $self.statistics.messages_sent = $self.statistics.messages_sent.saturating_add(1);
        }

        res
    }};
}

/// Aldrin broker.
///
/// This is the central message broker present in every Aldrin bus. After creating a `Broker` with
/// [`new`](Broker::new), it must be turned into future with [`run`](Broker::run) and then polled to
/// completion.
///
/// [`BrokerHandle`s](BrokerHandle) are used to interact with a running `Broker` and can be acquired
/// with the [`handle`](Broker::handle) method. Through a `BrokerHandle`, you can add new
/// connections to the `Broker` as well as shut it down again.
///
/// The `Broker` will automatically shut down, when there are no active connections and the last
/// `BrokerHandle` has been dropped.
///
/// # Examples
///
/// ```
/// use aldrin_broker::Broker;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new broker:
///     let broker = Broker::new();
///
///     // Acquire a BrokerHandle:
///     let mut handle = broker.handle().clone();
///
///     // Run the broker:
///     let join = tokio::spawn(broker.run());
///
///     // Add connections to the broker:
///     // ...
///
///     // Shut down the broker:
///     handle.shutdown().await;
///     join.await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
#[must_use = "brokers do nothing unless you `.await` or poll `Broker::run()`"]
pub struct Broker {
    recv: Receiver<ConnectionEvent>,
    handle: Option<BrokerHandle>,
    conns: HashMap<ConnectionId, ConnectionState>,
    obj_uuids: HashMap<ObjectCookie, ObjectUuid>,
    objs: HashMap<ObjectUuid, Object>,
    svc_uuids: HashMap<ServiceCookie, (ObjectId, ServiceUuid, ServiceInfo)>,
    svcs: HashMap<(ObjectUuid, ServiceUuid), Service>,
    function_calls: SerialMap<PendingFunctionCall>,
    channels: HashMap<ChannelCookie, Channel>,
    bus_listeners: HashMap<BusListenerCookie, BusListener>,
    #[cfg(feature = "statistics")]
    statistics: BrokerStatistics,
    #[cfg(feature = "introspection")]
    introspection: IntrospectionDatabase,
    #[cfg(feature = "introspection")]
    query_introspection: SerialMap<TypeId>,
}

impl Broker {
    /// Creates a new broker.
    ///
    /// After creating a `Broker`, it must be turned into a future with [`run`](Broker::run) and
    /// polled to completion.
    pub fn new() -> Self {
        let (send, recv) = channel(FIFO_SIZE);

        Self {
            recv,
            handle: Some(BrokerHandle::new(send)),
            conns: HashMap::new(),
            obj_uuids: HashMap::new(),
            objs: HashMap::new(),
            svc_uuids: HashMap::new(),
            svcs: HashMap::new(),
            function_calls: SerialMap::new(),
            channels: HashMap::new(),
            bus_listeners: HashMap::new(),
            #[cfg(feature = "statistics")]
            statistics: BrokerStatistics::new(),
            #[cfg(feature = "introspection")]
            introspection: IntrospectionDatabase::new(),
            #[cfg(feature = "introspection")]
            query_introspection: SerialMap::new(),
        }
    }

    /// Returns a reference to the broker handle.
    ///
    /// It is important to acquire at least one [`BrokerHandle`] before [`run`ning](Broker::run) the
    /// `Broker`. `BrokerHandle`s are the only way to add new connections to the `Broker`.
    ///
    /// Note also, that this method returns only a reference. However, `BrokerHandle`s are cheap to
    /// `clone`.
    pub fn handle(&self) -> &BrokerHandle {
        self.handle.as_ref().unwrap()
    }

    /// Runs the broker.
    ///
    /// This is a long running method, that will only return when explicitly shut down or when there
    /// are no connected clients and the last [`BrokerHandle`] has been dropped.
    ///
    /// Make sure to [acquire](Broker::handle) a `BrokerHandle` before running the `Broker`.
    pub async fn run(mut self) {
        self.handle.take().unwrap();

        let mut state = State::new();

        loop {
            if state.shutdown_now() || (state.shutdown_idle() && self.conns.is_empty()) {
                break;
            }

            let ev = match self.recv.next().await {
                Some(ev) => ev,
                None => return,
            };

            self.handle_event(&mut state, ev);
            self.process_loop_result(&mut state);
        }

        debug_assert!(!state.has_work_left());
        debug_assert!(self.conns.is_empty());
        debug_assert!(self.obj_uuids.is_empty());
        debug_assert!(self.objs.is_empty());
        debug_assert!(self.svc_uuids.is_empty());
        debug_assert!(self.svcs.is_empty());
        debug_assert!(self.function_calls.is_empty());
    }

    fn handle_event(&mut self, state: &mut State, ev: ConnectionEvent) {
        match ev {
            ConnectionEvent::NewConnection(id, protocol_version, sender) => {
                let dup = self
                    .conns
                    .insert(id, ConnectionState::new(protocol_version, sender));
                debug_assert!(dup.is_none());

                #[cfg(feature = "statistics")]
                {
                    self.statistics.num_connections =
                        self.statistics.num_connections.saturating_add(1);
                }
            }

            ConnectionEvent::ConnectionShutdown(id) => {
                state.push_remove_conn(id, false);
            }

            ConnectionEvent::Message(id, msg) => {
                if self.handle_message(state, &id, msg).is_err() {
                    state.push_remove_conn(id, false);
                }

                #[cfg(feature = "statistics")]
                {
                    self.statistics.messages_received =
                        self.statistics.messages_received.saturating_add(1);
                }
            }

            ConnectionEvent::ShutdownBroker => {
                state.push_remove_conns(self.conns.keys().cloned().map(|id| (id, true)));
                state.set_shutdown_now();
            }

            ConnectionEvent::ShutdownIdleBroker => {
                state.set_shutdown_idle();
            }

            ConnectionEvent::ShutdownConnection(id) => {
                state.push_remove_conn(id, true);
            }

            #[cfg(feature = "statistics")]
            ConnectionEvent::TakeStatistics(sender) => {
                let _ = sender.send(self.statistics.take());
            }
        }
    }

    fn process_loop_result(&mut self, state: &mut State) {
        loop {
            // The order in which events are processed and sent to clients matters here.
            // Always remove connections first. That way we never actually try to send events to
            // clients, which are known to be shut down.
            // Then, handle all "add" events before "remove" events. Otherwise we might announce new
            // objects and services, which have previously been declared destroyed.

            if let Some((conn_id, send_shutdown)) = state.pop_remove_conn() {
                self.shutdown_connection(state, &conn_id, send_shutdown);
                continue;
            }

            if let Some((conn_id, service_cookie, event)) = state.pop_unsubscribe_event() {
                let Some(conn) = self.conns.get(&conn_id) else {
                    continue;
                };

                let msg = UnsubscribeEvent {
                    service_cookie,
                    event,
                };

                if send!(self, conn, msg).is_err() {
                    state.push_remove_conn(conn_id, false);
                }

                continue;
            }

            if let Some((conn_id, service_cookie)) = state.pop_unsubscribe_all_events() {
                let Some(conn) = self.conns.get(&conn_id) else {
                    continue;
                };

                let msg = UnsubscribeAllEvents {
                    serial: None,
                    service_cookie,
                };

                if send!(self, conn, msg).is_err() {
                    state.push_remove_conn(conn_id, false);
                }

                continue;
            }

            if let Some((conn_id, service_cookie)) = state.pop_services_destroyed() {
                let Some(conn) = self.conns.get(&conn_id) else {
                    continue;
                };

                if send!(self, conn, ServiceDestroyed { service_cookie }).is_err() {
                    state.push_remove_conn(conn_id, false);
                }

                continue;
            }

            if let Some((serial, conn_id, result)) = state.pop_remove_function_call() {
                let Some(conn) = self.conns.get_mut(&conn_id) else {
                    continue;
                };

                conn.remove_call(serial);

                if send!(self, conn, CallFunctionReply { serial, result }).is_err() {
                    state.push_remove_conn(conn_id, false);
                }

                continue;
            }

            if let Some(object) = state.pop_create_object() {
                self.emit_bus_event(state, BusEvent::ObjectCreated(object));
                continue;
            }

            if let Some(service) = state.pop_create_service() {
                self.emit_bus_event(state, BusEvent::ServiceCreated(service));
                continue;
            }

            if let Some(service) = state.pop_destroy_service() {
                self.emit_bus_event(state, BusEvent::ServiceDestroyed(service));
                continue;
            }

            if let Some(object) = state.pop_destroy_object() {
                self.emit_bus_event(state, BusEvent::ObjectDestroyed(object));
                continue;
            }

            if let Some((callee_serial, callee_id)) = state.pop_abort_function_call() {
                self.abort_call(state, callee_serial, callee_id);
                continue;
            }

            debug_assert!(!state.has_work_left());
            break;
        }
    }

    fn shutdown_connection(&mut self, state: &mut State, id: &ConnectionId, send_shutdown: bool) {
        let Some(conn) = self.conns.remove(id) else {
            return;
        };

        if send_shutdown {
            // Ignore errors here.
            let _ = send!(self, conn, Shutdown);
        }

        for bus_listener_cookie in conn.bus_listeners() {
            self.remove_bus_listener(bus_listener_cookie);
        }

        for obj_cookie in conn.objects() {
            self.remove_object(state, obj_cookie);
        }

        for (svc_cookie, event) in conn.event_subscriptions() {
            self.remove_event_subscription(state, id, svc_cookie, event);
        }

        for svc_cookie in conn.all_event_subscriptions() {
            self.remove_all_events_subscription(state, id, svc_cookie);
        }

        for svc_cookie in conn.subscriptions() {
            self.remove_subscription(id, svc_cookie);
        }

        for chann_cookie in conn.senders() {
            self.remove_channel_end(state, chann_cookie, ChannelEnd::Sender, Some(id));
        }

        for chann_cookie in conn.receivers() {
            self.remove_channel_end(state, chann_cookie, ChannelEnd::Receiver, Some(id));
        }

        for (callee_serial, callee_id) in conn.calls() {
            state.push_abort_function_call(callee_serial, callee_id.clone());
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_connections = self.statistics.num_connections.saturating_sub(1);
        }

        #[cfg(feature = "introspection")]
        self.remove_introspection_conn(state, id);
    }

    fn handle_message(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        msg: Message,
    ) -> Result<(), ()> {
        match msg {
            Message::CreateObject(req) => self.create_object(state, id, req)?,
            Message::DestroyObject(req) => self.destroy_object(state, id, req)?,
            Message::CreateService(req) => self.create_service(state, id, req)?,
            Message::DestroyService(req) => self.destroy_service(state, id, req)?,
            Message::CallFunction(req) => self.call_function(state, id, req)?,
            Message::CallFunctionReply(req) => self.call_function_reply(state, id, req),
            Message::SubscribeEvent(req) => self.subscribe_event(id, req)?,
            Message::UnsubscribeEvent(req) => self.unsubscribe_event(state, id, req),
            Message::EmitEvent(req) => self.emit_event(state, id, req),
            Message::QueryServiceVersion(req) => self.query_service_version(id, req)?,
            Message::CreateChannel(req) => self.create_channel(id, req)?,
            Message::CloseChannelEnd(req) => self.close_channel_end(state, id, req)?,
            Message::ClaimChannelEnd(req) => self.claim_channel_end(state, id, req)?,
            Message::AddChannelCapacity(req) => self.add_channel_capacity(state, id, req),
            Message::SendItem(req) => self.send_item(state, id, req)?,
            Message::Sync(req) => self.sync(id, req)?,
            Message::CreateBusListener(req) => self.create_bus_listener(id, req)?,
            Message::DestroyBusListener(req) => self.destroy_bus_listener(id, req)?,
            Message::AddBusListenerFilter(req) => self.add_bus_listener_filter(id, req),
            Message::RemoveBusListenerFilter(req) => self.remove_bus_listener_filter(id, req),
            Message::ClearBusListenerFilters(req) => self.clear_bus_listener_filters(id, req),
            Message::StartBusListener(req) => self.start_bus_listener(id, req)?,
            Message::StopBusListener(req) => self.stop_bus_listener(id, req)?,
            Message::AbortFunctionCall(req) => self.abort_function_call(state, id, req)?,
            Message::RegisterIntrospection(req) => self.register_introspection(id, req)?,
            Message::QueryIntrospection(req) => self.query_introspection(state, id, req)?,
            Message::QueryIntrospectionReply(req) => {
                self.query_introspection_reply(state, id, req)?
            }
            Message::CreateService2(req) => self.create_service2(state, id, req)?,
            Message::QueryServiceInfo(req) => self.query_service_info(id, req)?,
            Message::SubscribeService(req) => self.subscribe_service(id, req)?,
            Message::UnsubscribeService(req) => self.unsubscribe_service(id, req)?,
            Message::SubscribeAllEvents(req) => self.subscribe_all_events(id, req)?,
            Message::UnsubscribeAllEvents(req) => self.unsubscribe_all_events(id, req)?,

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObjectReply(_)
            | Message::DestroyObjectReply(_)
            | Message::CreateServiceReply(_)
            | Message::DestroyServiceReply(_)
            | Message::SubscribeEventReply(_)
            | Message::QueryServiceVersionReply(_)
            | Message::CreateChannelReply(_)
            | Message::CloseChannelEndReply(_)
            | Message::ChannelEndClosed(_)
            | Message::ClaimChannelEndReply(_)
            | Message::ChannelEndClaimed(_)
            | Message::ItemReceived(_)
            | Message::SyncReply(_)
            | Message::ServiceDestroyed(_)
            | Message::CreateBusListenerReply(_)
            | Message::DestroyBusListenerReply(_)
            | Message::StartBusListenerReply(_)
            | Message::StopBusListenerReply(_)
            | Message::EmitBusEvent(_)
            | Message::BusListenerCurrentFinished(_)
            | Message::Connect2(_)
            | Message::ConnectReply2(_)
            | Message::QueryServiceInfoReply(_)
            | Message::SubscribeServiceReply(_)
            | Message::SubscribeAllEventsReply(_)
            | Message::UnsubscribeAllEventsReply(_) => return Err(()),

            Message::Shutdown(Shutdown) => unreachable!(), // Handled by connection.
        }

        Ok(())
    }

    fn create_object(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CreateObject,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        match self.objs.entry(req.uuid) {
            Entry::Occupied(_) => send!(
                self,
                conn,
                CreateObjectReply {
                    serial: req.serial,
                    result: CreateObjectResult::DuplicateObject,
                },
            ),

            Entry::Vacant(entry) => {
                let cookie = ObjectCookie::new_v4();

                send!(
                    self,
                    conn,
                    CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::Ok(cookie),
                    },
                )?;

                let dup = self.obj_uuids.insert(cookie, req.uuid);
                debug_assert!(dup.is_none());
                entry.insert(Object::new(id.clone()));
                conn.add_object(cookie);
                state.push_create_object(ObjectId::new(req.uuid, cookie));

                #[cfg(feature = "statistics")]
                {
                    self.statistics.num_objects = self.statistics.num_objects.saturating_add(1);
                }

                Ok(())
            }
        }
    }

    fn destroy_object(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: DestroyObject,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(&obj_uuid) = self.obj_uuids.get(&req.cookie) else {
            return send!(
                self,
                conn,
                DestroyObjectReply {
                    serial: req.serial,
                    result: DestroyObjectResult::InvalidObject,
                },
            );
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");

        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                DestroyObjectReply {
                    serial: req.serial,
                    result: DestroyObjectResult::ForeignObject,
                },
            );
        }

        send!(
            self,
            conn,
            DestroyObjectReply {
                serial: req.serial,
                result: DestroyObjectResult::Ok,
            },
        )?;

        self.remove_object(state, req.cookie);
        Ok(())
    }

    fn create_service(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CreateService,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(&obj_uuid) = self.obj_uuids.get(&req.object_cookie) else {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::InvalidObject,
                },
            );
        };

        let Entry::Vacant(entry) = self.svcs.entry((obj_uuid, req.uuid)) else {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::DuplicateService,
                },
            );
        };

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::ForeignObject,
                },
            );
        }

        let svc_cookie = ServiceCookie::new_v4();

        send!(
            self,
            conn,
            CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::Ok(svc_cookie),
            },
        )?;

        let object_id = ObjectId::new(obj_uuid, req.object_cookie);
        let info = ServiceInfo::new(req.version);
        let dup = self
            .svc_uuids
            .insert(svc_cookie, (object_id, req.uuid, info));
        debug_assert!(dup.is_none());
        entry.insert(Service::new());
        obj.add_service(svc_cookie);
        state.push_create_service(ServiceId::new(object_id, req.uuid, svc_cookie));

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_services = self.statistics.num_services.saturating_add(1);
        }

        Ok(())
    }

    fn destroy_service(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: DestroyService,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(ids) = self.svc_uuids.get(&req.cookie) else {
            return send!(
                self,
                conn,
                DestroyServiceReply {
                    serial: req.serial,
                    result: DestroyServiceResult::InvalidService,
                },
            );
        };

        let obj = self.objs.get(&ids.0.uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                DestroyServiceReply {
                    serial: req.serial,
                    result: DestroyServiceResult::ForeignObject,
                },
            );
        }

        send!(
            self,
            conn,
            DestroyServiceReply {
                serial: req.serial,
                result: DestroyServiceResult::Ok,
            },
        )?;

        self.remove_service(state, req.cookie);
        Ok(())
    }

    fn call_function(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CallFunction,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&req.service_cookie) else {
            return send!(
                self,
                conn,
                CallFunctionReply {
                    serial: req.serial,
                    result: CallFunctionResult::InvalidService,
                },
            );
        };

        let callee_id = self
            .objs
            .get(&obj_id.uuid)
            .expect("inconsistent state")
            .conn_id();

        let serial = self.function_calls.insert(PendingFunctionCall {
            caller_serial: req.serial,
            caller_conn_id: id.clone(),
            callee_obj: obj_id.uuid,
            callee_svc: svc_uuid,
            aborted: false,
        });

        if !conn.add_call(req.serial, serial, callee_id.clone()) {
            self.function_calls.remove(serial);
            return Err(());
        }

        let callee_conn = self.conns.get(callee_id).expect("inconsistent state");

        self.svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state")
            .add_function_call(serial);

        let res = send!(
            self,
            callee_conn,
            CallFunction {
                serial,
                service_cookie: req.service_cookie,
                function: req.function,
                value: req.value,
            },
        );

        if res.is_err() {
            state.push_remove_conn(callee_id.clone(), false);
        }

        Ok(())
    }

    fn call_function_reply(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CallFunctionReply,
    ) {
        let Some(call) = self.function_calls.get(req.serial) else {
            return;
        };

        let obj = self.objs.get(&call.callee_obj).expect("inconsistent state");
        if obj.conn_id() != id {
            return;
        }

        let call = self.function_calls.remove(req.serial).unwrap();

        let svc = self
            .svcs
            .get_mut(&(call.callee_obj, call.callee_svc))
            .expect("inconsistent state");

        svc.remove_function_call(req.serial);

        if call.aborted {
            return;
        }

        let Some(conn) = self.conns.get_mut(&call.caller_conn_id) else {
            return;
        };

        conn.remove_call(call.caller_serial);

        let res = send!(
            self,
            conn,
            CallFunctionReply {
                serial: call.caller_serial,
                result: req.result,
            },
        );

        if res.is_err() {
            state.push_remove_conn(call.caller_conn_id, false);
        }
    }

    fn subscribe_event(&mut self, id: &ConnectionId, req: SubscribeEvent) -> Result<(), ()> {
        let Some(serial) = req.serial else {
            return Err(());
        };

        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&req.service_cookie) else {
            return send!(
                self,
                conn,
                SubscribeEventReply {
                    serial,
                    result: SubscribeEventResult::InvalidService,
                },
            );
        };

        send!(
            self,
            conn,
            SubscribeEventReply {
                serial,
                result: SubscribeEventResult::Ok,
            },
        )?;

        conn.subscribe_event(req.service_cookie, req.event);
        let send_req = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state")
            .subscribe_event(req.event, id.clone());

        if send_req {
            let target_conn_id = self
                .objs
                .get_mut(&obj_id.uuid)
                .expect("inconsistent state")
                .conn_id();

            if let Some(target_conn) = self.conns.get(target_conn_id) {
                let _ = send!(
                    self,
                    target_conn,
                    SubscribeEvent {
                        serial: None,
                        service_cookie: req.service_cookie,
                        event: req.event,
                    },
                );
            }
        }

        Ok(())
    }

    fn unsubscribe_event(&mut self, state: &mut State, id: &ConnectionId, req: UnsubscribeEvent) {
        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&req.service_cookie) else {
            return;
        };

        let svc = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state");

        let Some(conn) = self.conns.get_mut(id) else {
            return;
        };

        conn.unsubscribe_event(req.service_cookie, req.event);
        let send_unsubscribe = svc.unsubscribe_event(req.event, id);

        if send_unsubscribe {
            let obj = self.objs.get(&obj_id.uuid).expect("inconsistent state");
            let conn_id = obj.conn_id();
            let conn = self.conns.get(conn_id).expect("inconsistent state");

            if send!(self, conn, req).is_err() {
                state.push_remove_conn(conn_id.clone(), false);
            }
        }
    }

    fn emit_event(&mut self, state: &mut State, id: &ConnectionId, req: EmitEvent) {
        let Some(obj_uuid) = self
            .svc_uuids
            .get(&req.service_cookie)
            .map(|(object_id, _, _)| object_id.uuid)
        else {
            return;
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return;
        }

        for (conn_id, conn) in self.conns.iter() {
            if conn.is_subscribed_to_event(req.service_cookie, req.event)
                && send!(self, conn, req.clone()).is_err()
            {
                state.push_remove_conn(conn_id.clone(), false);
            }
        }
    }

    fn query_service_version(
        &mut self,
        id: &ConnectionId,
        req: QueryServiceVersion,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let reply = match self.svc_uuids.get(&req.cookie) {
            Some(&(_, _, info)) => QueryServiceVersionReply {
                serial: req.serial,
                result: QueryServiceVersionResult::Ok(info.version()),
            },

            None => QueryServiceVersionReply {
                serial: req.serial,
                result: QueryServiceVersionResult::InvalidService,
            },
        };

        send!(self, conn, reply)
    }

    fn create_channel(&mut self, id: &ConnectionId, req: CreateChannel) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        let cookie = ChannelCookie::new_v4();

        let channel = match req.end {
            ChannelEndWithCapacity::Sender => {
                conn.add_sender(cookie);
                Channel::with_claimed_sender(id.clone())
            }

            ChannelEndWithCapacity::Receiver(capacity) => {
                conn.add_receiver(cookie);
                Channel::with_claimed_receiver(id.clone(), capacity)
            }
        };

        self.channels.insert(cookie, channel);

        send!(
            self,
            conn,
            CreateChannelReply {
                serial: req.serial,
                cookie,
            },
        )?;

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_channels = self.statistics.num_channels.saturating_add(1);
        }

        Ok(())
    }

    fn close_channel_end(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CloseChannelEnd,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(channel) = self.channels.get(&req.cookie) else {
            return send!(
                self,
                conn,
                CloseChannelEndReply {
                    serial: req.serial,
                    result: CloseChannelEndResult::InvalidChannel,
                },
            );
        };

        let (result, claimed) = channel.check_close(id, req.end);

        send!(
            self,
            conn,
            CloseChannelEndReply {
                serial: req.serial,
                result,
            },
        )?;

        if result == CloseChannelEndResult::Ok {
            let owner = if claimed { Some(id) } else { None };
            self.remove_channel_end(state, req.cookie, req.end, owner);
        }

        Ok(())
    }

    fn claim_channel_end(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: ClaimChannelEnd,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        let Some(channel) = self.channels.get_mut(&req.cookie) else {
            return send!(
                self,
                conn,
                ClaimChannelEndReply {
                    serial: req.serial,
                    result: ClaimChannelEndResult::InvalidChannel,
                },
            );
        };

        let result = match req.end {
            ChannelEndWithCapacity::Sender => {
                channel.claim_sender(id).map(|(receiver, capacity)| {
                    conn.add_sender(req.cookie);
                    (receiver, ClaimChannelEndResult::SenderClaimed(capacity))
                })
            }

            ChannelEndWithCapacity::Receiver(capacity) => {
                channel.claim_receiver(id, capacity).map(|sender| {
                    conn.add_receiver(req.cookie);
                    (sender, ClaimChannelEndResult::ReceiverClaimed)
                })
            }
        };

        match result {
            Ok((other_id, result)) => {
                let result = send!(
                    self,
                    conn,
                    ClaimChannelEndReply {
                        serial: req.serial,
                        result,
                    },
                );

                let other = self.conns.get_mut(other_id).expect("inconsistent state");

                let other_result = send!(
                    self,
                    other,
                    ChannelEndClaimed {
                        cookie: req.cookie,
                        end: req.end,
                    },
                );

                if other_result.is_err() {
                    state.push_remove_conn(other_id.clone(), false);
                }

                result
            }

            Err(result) => send!(
                self,
                conn,
                ClaimChannelEndReply {
                    serial: req.serial,
                    result,
                },
            ),
        }
    }

    fn add_channel_capacity(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: AddChannelCapacity,
    ) {
        let Some(channel) = self.channels.get_mut(&req.cookie) else {
            return;
        };

        let (sender_id, capacity) = match channel.add_capacity(id, req.capacity) {
            Ok(Some((sender_id, capacity))) => (sender_id, capacity),
            Ok(None) => return,

            Err(AddCapacityError) => {
                self.remove_channel_end(state, req.cookie, ChannelEnd::Receiver, Some(id));
                return;
            }
        };

        let Some(sender) = self.conns.get(sender_id) else {
            return;
        };

        let res = send!(
            self,
            sender,
            AddChannelCapacity {
                cookie: req.cookie,
                capacity,
            },
        );

        if res.is_err() {
            state.push_remove_conn(sender_id.clone(), false);
        }
    }

    fn send_item(&mut self, state: &mut State, id: &ConnectionId, req: SendItem) -> Result<(), ()> {
        let Some(sender) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(channel) = self.channels.get_mut(&req.cookie) else {
            return Ok(());
        };

        let (receiver_id, add_capacity) = match channel.send_item(id) {
            Ok(res) => res,

            Err(e) => {
                match e {
                    SendItemError::ReceiverUnclaimed => {
                        // The order matters here. The sender needs to get a notification that the
                        // receiver was closed. But, if we closed the sender first, then this would
                        // remove the whole channel because the receiver isn't claimed. In that
                        // case, the closing the receiver afterwards would become a no-op. Hence,
                        // close the receiver first.
                        self.remove_channel_end(state, req.cookie, ChannelEnd::Receiver, None);
                        self.remove_channel_end(state, req.cookie, ChannelEnd::Sender, Some(id));
                    }

                    SendItemError::CapacityExhausted => {
                        self.remove_channel_end(state, req.cookie, ChannelEnd::Sender, Some(id))
                    }

                    SendItemError::InvalidSender | SendItemError::ReceiverClosed => {}
                }

                return Ok(());
            }
        };

        let Some(receiver) = self.conns.get(receiver_id) else {
            return Ok(());
        };

        let res = send!(
            self,
            receiver,
            ItemReceived {
                cookie: req.cookie,
                value: req.value,
            },
        );

        if res.is_err() {
            state.push_remove_conn(receiver_id.clone(), false);
        }

        if let Some(add_capacity) = add_capacity {
            send!(
                self,
                sender,
                AddChannelCapacity {
                    cookie: req.cookie,
                    capacity: add_capacity,
                },
            )
        } else {
            Ok(())
        }
    }

    fn sync(&mut self, id: &ConnectionId, req: Sync) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        send!(self, conn, SyncReply { serial: req.serial })
    }

    fn create_bus_listener(&mut self, id: &ConnectionId, req: CreateBusListener) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        let cookie = BusListenerCookie::new_v4();

        send!(
            self,
            conn,
            CreateBusListenerReply {
                serial: req.serial,
                cookie,
            },
        )?;

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_bus_listeners = self.statistics.num_bus_listeners.saturating_add(1);
        }

        conn.add_bus_listener(cookie);
        self.bus_listeners
            .insert(cookie, BusListener::new(id.clone()));

        Ok(())
    }

    fn destroy_bus_listener(
        &mut self,
        id: &ConnectionId,
        req: DestroyBusListener,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(bus_listener) = self.bus_listeners.get(&req.cookie) else {
            return send!(
                self,
                conn,
                DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::InvalidBusListener,
                },
            );
        };

        if bus_listener.conn_id() == id {
            send!(
                self,
                conn,
                DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::Ok,
                },
            )?;

            self.remove_bus_listener(req.cookie);
        } else {
            send!(
                self,
                conn,
                DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::InvalidBusListener,
                },
            )?;
        }

        Ok(())
    }

    fn add_bus_listener_filter(&mut self, id: &ConnectionId, req: AddBusListenerFilter) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.add_filter(req.filter);
            }
        }
    }

    fn remove_bus_listener_filter(&mut self, id: &ConnectionId, req: RemoveBusListenerFilter) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.remove_filter(req.filter);
            }
        }
    }

    fn clear_bus_listener_filters(&mut self, id: &ConnectionId, req: ClearBusListenerFilters) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.clear_filters();
            }
        }
    }

    fn start_bus_listener(&mut self, id: &ConnectionId, req: StartBusListener) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
            return send!(
                self,
                conn,
                StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::InvalidBusListener,
                },
            );
        };

        if bus_listener.conn_id() != id {
            return send!(
                self,
                conn,
                StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::InvalidBusListener,
                },
            );
        }

        if !bus_listener.start(req.scope) {
            return send!(
                self,
                conn,
                StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::AlreadyStarted,
                },
            );
        }

        send!(
            self,
            conn,
            StartBusListenerReply {
                serial: req.serial,
                result: StartBusListenerResult::Ok,
            },
        )?;

        if req.scope != BusListenerScope::New {
            for (&cookie, &uuid) in &self.obj_uuids {
                let object = ObjectId::new(uuid, cookie);

                if bus_listener.matches_object(object) {
                    send!(
                        self,
                        conn,
                        EmitBusEvent {
                            cookie: Some(req.cookie),
                            event: BusEvent::ObjectCreated(object),
                        },
                    )?;
                }
            }

            for (&service_cookie, &(object, service_uuid, _)) in &self.svc_uuids {
                let service = ServiceId::new(object, service_uuid, service_cookie);

                if bus_listener.matches_service(service) {
                    send!(
                        self,
                        conn,
                        EmitBusEvent {
                            cookie: Some(req.cookie),
                            event: BusEvent::ServiceCreated(service),
                        },
                    )?;
                }
            }

            send!(
                self,
                conn,
                BusListenerCurrentFinished { cookie: req.cookie },
            )?;
        }

        Ok(())
    }

    fn stop_bus_listener(&mut self, id: &ConnectionId, req: StopBusListener) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) else {
            return send!(
                self,
                conn,
                StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::InvalidBusListener,
                },
            );
        };

        if bus_listener.conn_id() != id {
            return send!(
                self,
                conn,
                StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::InvalidBusListener,
                },
            );
        }

        if bus_listener.stop() {
            send!(
                self,
                conn,
                StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::Ok,
                },
            )
        } else {
            send!(
                self,
                conn,
                StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::NotStarted,
                },
            )
        }
    }

    fn abort_function_call(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: AbortFunctionCall,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_16 {
            return Err(());
        }

        let Some((callee_serial, callee_id)) = conn.call_data(req.serial) else {
            return Ok(());
        };

        state.push_abort_function_call(callee_serial, callee_id.clone());
        Ok(())
    }

    #[cfg(feature = "introspection")]
    fn register_introspection(
        &mut self,
        id: &ConnectionId,
        req: RegisterIntrospection,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        if let Ok(type_ids) = req.deserialize_type_ids() {
            self.introspection.register(&type_ids, id);

            #[cfg(feature = "statistics")]
            {
                self.statistics.num_introspections = self.introspection.len();
            }

            Ok(())
        } else {
            Err(())
        }
    }

    #[cfg(not(feature = "introspection"))]
    fn register_introspection(
        &self,
        id: &ConnectionId,
        _req: RegisterIntrospection,
    ) -> Result<(), ()> {
        if let Some(conn) = self.conns.get(id) {
            if conn.protocol_version() >= ProtocolVersion::V1_17 {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "introspection")]
    fn query_introspection(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: QueryIntrospection,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        let Some(entry) = self.introspection.get_mut(req.type_id) else {
            return send!(
                self,
                conn,
                QueryIntrospectionReply {
                    serial: req.serial,
                    result: QueryIntrospectionResult::Unavailable,
                },
            );
        };

        if let Some(introspection) = entry.introspection() {
            send!(
                self,
                conn,
                QueryIntrospectionReply {
                    serial: req.serial,
                    result: QueryIntrospectionResult::Ok(introspection.clone()),
                },
            )
        } else {
            entry.add_pending(id.clone(), req.serial);

            if entry.queried().is_none() {
                let serial = self.query_introspection.insert(req.type_id);
                let conn_id = entry.query_random_conn(serial);
                let conn = self.conns.get(conn_id).expect("inconsistent state");

                let msg = QueryIntrospection {
                    serial,
                    type_id: req.type_id,
                };

                if send!(self, conn, msg).is_err() {
                    state.push_remove_conn(conn_id.clone(), false);
                }
            }

            Ok(())
        }
    }

    #[cfg(not(feature = "introspection"))]
    fn query_introspection(
        &mut self,
        _state: &mut State,
        id: &ConnectionId,
        req: QueryIntrospection,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        send!(
            self,
            conn,
            QueryIntrospectionReply {
                serial: req.serial,
                result: QueryIntrospectionResult::Unavailable,
            },
        )
    }

    #[cfg(feature = "introspection")]
    fn query_introspection_reply(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: QueryIntrospectionReply,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        let serial = req.serial;

        let Some(&type_id) = self.query_introspection.get(serial) else {
            return Err(());
        };

        let Some(res) = self.introspection.query_replied(type_id, id, req) else {
            #[cfg(feature = "statistics")]
            {
                self.statistics.num_introspections = self.introspection.len();
            }

            return Err(());
        };

        self.query_introspection.remove(serial);

        match res {
            IntrospectionQueryResult::Available {
                introspection,
                pending,
            } => {
                for pending in pending {
                    let conn = self
                        .conns
                        .get(&pending.conn_id)
                        .expect("inconsistent state");

                    let msg = QueryIntrospectionReply {
                        serial: pending.serial,
                        result: QueryIntrospectionResult::Ok(introspection.clone()),
                    };

                    if send!(self, conn, msg).is_err() {
                        state.push_remove_conn(pending.conn_id, false);
                    }
                }
            }

            IntrospectionQueryResult::Unavailable(pending) => {
                for pending in pending {
                    let conn = self
                        .conns
                        .get(&pending.conn_id)
                        .expect("inconsistent state");

                    let msg = QueryIntrospectionReply {
                        serial: pending.serial,
                        result: QueryIntrospectionResult::Unavailable,
                    };

                    if send!(self, conn, msg).is_err() {
                        state.push_remove_conn(pending.conn_id, false);
                    }
                }
            }

            IntrospectionQueryResult::Continue(entry) => {
                let serial = self.query_introspection.insert(type_id);
                let conn_id = entry.query_random_conn(serial);
                let conn = self.conns.get(conn_id).expect("inconsistent state");
                let msg = QueryIntrospection { serial, type_id };

                if send!(self, conn, msg).is_err() {
                    state.push_remove_conn(conn_id.clone(), false);
                }
            }
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_introspections = self.introspection.len();
        }

        Ok(())
    }

    #[cfg(not(feature = "introspection"))]
    fn query_introspection_reply(
        &mut self,
        _state: &mut State,
        _id: &ConnectionId,
        _req: QueryIntrospectionReply,
    ) -> Result<(), ()> {
        Err(())
    }

    fn create_service2(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CreateService2,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        let Some(&obj_uuid) = self.obj_uuids.get(&req.object_cookie) else {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::InvalidObject,
                },
            );
        };

        let Entry::Vacant(entry) = self.svcs.entry((obj_uuid, req.uuid)) else {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::DuplicateService,
                },
            );
        };

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::ForeignObject,
                },
            );
        }

        let Ok(mut info) = req.deserialize_info() else {
            return Err(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_18 {
            info = info.set_subscribe_all(false);
        }

        let svc_cookie = ServiceCookie::new_v4();
        send!(
            self,
            conn,
            CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::Ok(svc_cookie),
            },
        )?;

        let object_id = ObjectId::new(obj_uuid, req.object_cookie);
        let dup = self
            .svc_uuids
            .insert(svc_cookie, (object_id, req.uuid, info));
        debug_assert!(dup.is_none());
        entry.insert(Service::new());
        obj.add_service(svc_cookie);
        state.push_create_service(ServiceId::new(object_id, req.uuid, svc_cookie));

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_services = self.statistics.num_services.saturating_add(1);
        }

        Ok(())
    }

    fn query_service_info(&mut self, id: &ConnectionId, req: QueryServiceInfo) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_17 {
            return Err(());
        }

        let reply = match self.svc_uuids.get(&req.cookie) {
            Some(&(_, _, info)) => QueryServiceInfoReply::ok_with_serialize_info(req.serial, info)
                .expect("failed to serialize ServiceInfo"),

            None => QueryServiceInfoReply {
                serial: req.serial,
                result: QueryServiceInfoResult::InvalidService,
            },
        };

        send!(self, conn, reply)
    }

    fn subscribe_service(&mut self, id: &ConnectionId, req: SubscribeService) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_18 {
            return Err(());
        }

        match self.svc_uuids.get(&req.service_cookie) {
            Some(&(obj_id, svc_uuid, _)) => {
                send!(
                    self,
                    conn,
                    SubscribeServiceReply {
                        serial: req.serial,
                        result: SubscribeServiceResult::Ok,
                    },
                )?;

                self.svcs
                    .get_mut(&(obj_id.uuid, svc_uuid))
                    .expect("inconsistent state")
                    .subscribe(id.clone());

                conn.subscribe(req.service_cookie);

                Ok(())
            }

            None => {
                send!(
                    self,
                    conn,
                    SubscribeServiceReply {
                        serial: req.serial,
                        result: SubscribeServiceResult::InvalidService,
                    },
                )
            }
        }
    }

    fn unsubscribe_service(
        &mut self,
        id: &ConnectionId,
        req: UnsubscribeService,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get_mut(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_18 {
            return Err(());
        }

        if let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&req.service_cookie) {
            self.svcs
                .get_mut(&(obj_id.uuid, svc_uuid))
                .expect("inconsistent state")
                .unsubscribe(id);

            conn.unsubscribe(req.service_cookie);
        }

        Ok(())
    }

    fn subscribe_all_events(
        &mut self,
        id: &ConnectionId,
        req: SubscribeAllEvents,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_18 {
            return Err(());
        }

        let Some(serial) = req.serial else {
            return Err(());
        };

        let Some(&(obj_id, svc_uuid, info)) = self.svc_uuids.get(&req.service_cookie) else {
            return send!(
                self,
                conn,
                SubscribeAllEventsReply {
                    serial,
                    result: SubscribeAllEventsResult::InvalidService,
                },
            );
        };

        if !info.subscribe_all().unwrap_or(false) {
            return send!(
                self,
                conn,
                SubscribeAllEventsReply {
                    serial,
                    result: SubscribeAllEventsResult::NotSupported,
                },
            );
        }

        let target_conn_id = self
            .objs
            .get(&obj_id.uuid)
            .expect("inconsistent state")
            .conn_id();

        let target_conn = self.conns.get(target_conn_id).expect("inconsistent state");

        if target_conn.protocol_version() < ProtocolVersion::V1_18 {
            return send!(
                self,
                conn,
                SubscribeAllEventsReply {
                    serial,
                    result: SubscribeAllEventsResult::NotSupported,
                },
            );
        }

        send!(
            self,
            conn,
            SubscribeAllEventsReply {
                serial,
                result: SubscribeAllEventsResult::Ok,
            },
        )?;

        let conn = self.conns.get_mut(id).unwrap();

        conn.subscribe_all_events(req.service_cookie);
        let send_req = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state")
            .subscribe_all_events(id.clone());

        if send_req {
            let _ = send!(
                self,
                self.conns.get(target_conn_id).unwrap(),
                SubscribeAllEvents {
                    serial: None,
                    service_cookie: req.service_cookie,
                },
            );
        }

        Ok(())
    }

    fn unsubscribe_all_events(
        &mut self,
        id: &ConnectionId,
        req: UnsubscribeAllEvents,
    ) -> Result<(), ()> {
        let Some(conn) = self.conns.get(id) else {
            return Ok(());
        };

        if conn.protocol_version() < ProtocolVersion::V1_18 {
            return Err(());
        }

        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&req.service_cookie) else {
            if let Some(serial) = req.serial {
                return send!(
                    self,
                    conn,
                    UnsubscribeAllEventsReply {
                        serial,
                        result: UnsubscribeAllEventsResult::InvalidService,
                    },
                );
            } else {
                return Ok(());
            }
        };

        let target_conn_id = self
            .objs
            .get(&obj_id.uuid)
            .expect("inconsistent state")
            .conn_id();

        let target_conn = self.conns.get(target_conn_id).expect("inconsistent state");

        if target_conn.protocol_version() < ProtocolVersion::V1_18 {
            if let Some(serial) = req.serial {
                return send!(
                    self,
                    conn,
                    UnsubscribeAllEventsReply {
                        serial,
                        result: UnsubscribeAllEventsResult::NotSupported,
                    },
                );
            } else {
                return Ok(());
            }
        }

        if let Some(serial) = req.serial {
            send!(
                self,
                conn,
                UnsubscribeAllEventsReply {
                    serial,
                    result: UnsubscribeAllEventsResult::Ok,
                },
            )?;
        }

        let conn = self.conns.get_mut(id).unwrap();

        conn.unsubscribe_all_events(req.service_cookie);
        let send_req = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state")
            .unsubscribe_all_events(id);

        if send_req {
            let _ = send!(
                self,
                self.conns.get(target_conn_id).unwrap(),
                UnsubscribeAllEvents {
                    serial: None,
                    service_cookie: req.service_cookie,
                },
            );
        }

        Ok(())
    }

    /// Removes the object `obj_cookie` and queues up events in `state`.
    ///
    /// This function will also remove all services owned by that object as well as everything
    /// related (e.g. pending function calls). It is safe to call with an invalid `obj_cookie`.
    fn remove_object(&mut self, state: &mut State, obj_cookie: ObjectCookie) {
        let Some(obj_uuid) = self.obj_uuids.remove(&obj_cookie) else {
            return;
        };

        let obj = self.objs.remove(&obj_uuid).expect("inconsistent state");

        // The connection might already have been removed. E.g. when this function is called by
        // `shutdown_connection`.
        if let Some(conn) = self.conns.get_mut(obj.conn_id()) {
            conn.remove_object(obj_cookie);
        }

        state.push_destroy_object(ObjectId::new(obj_uuid, obj_cookie));

        for svc_cookie in obj.services() {
            self.remove_service(state, svc_cookie);
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_objects = self.statistics.num_objects.saturating_sub(1);
        }
    }

    /// Removes the service `svc_cookie` and queues up events in `state`.
    ///
    /// This function will also remove everything related to `svc_cookie`, e.g. pending function
    /// calls. It is safe to call with an invalid `svc_cookie`.
    fn remove_service(&mut self, state: &mut State, svc_cookie: ServiceCookie) {
        let Some((obj_id, svc_uuid, _)) = self.svc_uuids.remove(&svc_cookie) else {
            return;
        };

        let svc = self
            .svcs
            .remove(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state");

        // The object might already have been removed.
        if let Some(obj) = self.objs.get_mut(&obj_id.uuid) {
            obj.remove_service(svc_cookie);
        }

        state.push_destroy_service(ServiceId::new(obj_id, svc_uuid, svc_cookie));

        for serial in svc.function_calls() {
            let call = self
                .function_calls
                .remove(serial)
                .expect("inconsistent state");

            if !call.aborted {
                state.push_remove_function_call(
                    call.caller_serial,
                    call.caller_conn_id,
                    CallFunctionResult::InvalidService,
                );
            }
        }

        for conn_id in svc.subscribed_conn_ids() {
            if let Some(conn) = self.conns.get_mut(conn_id) {
                conn.unsubscribe_all(svc_cookie);
                state.push_services_destroyed(conn_id.clone(), svc_cookie);
            }
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_services = self.statistics.num_services.saturating_sub(1);
        }
    }

    /// Removes subscription of `event` on service `svc_cookie` made by connection `conn_id`.
    ///
    /// If this was the last subscription of `event`, a notification is sent to the owner of
    /// `svc_cookie`. It is safe to call this function with an invalid `conn_id` or `svc_cookie`.
    fn remove_event_subscription(
        &mut self,
        state: &mut State,
        conn_id: &ConnectionId,
        svc_cookie: ServiceCookie,
        event: u32,
    ) {
        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&svc_cookie) else {
            return;
        };

        // The connection might already have been removed.
        if let Some(conn) = self.conns.get_mut(conn_id) {
            conn.unsubscribe_event(svc_cookie, event);
        }

        let svc = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state");

        if svc.unsubscribe_event(event, conn_id) {
            let obj = self.objs.get(&obj_id.uuid).expect("inconsistent state");
            state.push_unsubscribe_event(obj.conn_id().clone(), svc_cookie, event);
        }
    }

    fn remove_all_events_subscription(
        &mut self,
        state: &mut State,
        conn_id: &ConnectionId,
        svc_cookie: ServiceCookie,
    ) {
        let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&svc_cookie) else {
            return;
        };

        // The connection might already have been removed.
        if let Some(conn) = self.conns.get_mut(conn_id) {
            conn.unsubscribe_all_events(svc_cookie);
        }

        let svc = self
            .svcs
            .get_mut(&(obj_id.uuid, svc_uuid))
            .expect("inconsistent state");

        if svc.unsubscribe_all_events(conn_id) {
            let obj = self.objs.get(&obj_id.uuid).expect("inconsistent state");
            state.push_unsubscribe_all_events(obj.conn_id().clone(), svc_cookie);
        }
    }

    fn remove_subscription(&mut self, conn_id: &ConnectionId, svc_cookie: ServiceCookie) {
        if let Some(&(obj_id, svc_uuid, _)) = self.svc_uuids.get(&svc_cookie) {
            self.svcs
                .get_mut(&(obj_id.uuid, svc_uuid))
                .expect("inconsistent state")
                .unsubscribe(conn_id);
        }
    }

    /// Removes a channel end and may notify the other side. Possibly also removes the entire
    /// channel.
    fn remove_channel_end(
        &mut self,
        state: &mut State,
        cookie: ChannelCookie,
        end: ChannelEnd,
        owner: Option<&ConnectionId>,
    ) {
        let Entry::Occupied(mut channel) = self.channels.entry(cookie) else {
            return;
        };

        if let Some(conn) = owner.and_then(|conn_id| self.conns.get_mut(conn_id)) {
            match end {
                ChannelEnd::Sender => conn.remove_sender(cookie),
                ChannelEnd::Receiver => conn.remove_receiver(cookie),
            }
        }

        let remove = match channel.get_mut().close(end) {
            Some(other_id) => match self.conns.get(other_id) {
                Some(other) => {
                    if send!(self, other, ChannelEndClosed { cookie, end }).is_err() {
                        state.push_remove_conn(other_id.clone(), false);
                    }

                    false
                }

                None => true,
            },

            None => true,
        };

        if remove {
            channel.remove();

            #[cfg(feature = "statistics")]
            {
                self.statistics.num_channels = self.statistics.num_channels.saturating_sub(1);
            }
        }
    }

    fn remove_bus_listener(&mut self, cookie: BusListenerCookie) {
        let Some(bus_listener) = self.bus_listeners.remove(&cookie) else {
            return;
        };

        if let Some(conn) = self.conns.get_mut(bus_listener.conn_id()) {
            conn.remove_bus_listener(cookie);
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_bus_listeners = self.statistics.num_bus_listeners.saturating_sub(1);
        }
    }

    fn emit_bus_event(&mut self, state: &mut State, event: BusEvent) {
        #[allow(clippy::mutable_key_type)]
        let mut dups = HashSet::new();

        #[allow(clippy::mutable_key_type)]
        let mut remove_conns = HashSet::new();

        for bus_listener in self.bus_listeners.values() {
            let conn_id = bus_listener.conn_id();

            if !bus_listener.matches_new_event(event) || dups.contains(conn_id) {
                continue;
            }

            dups.insert(conn_id);

            let Some(conn) = self.conns.get(conn_id) else {
                continue;
            };

            let res = send!(
                self,
                conn,
                EmitBusEvent {
                    cookie: None,
                    event,
                },
            );

            if res.is_err() {
                remove_conns.insert(conn_id);
            }
        }

        state.push_remove_conns(remove_conns.into_iter().map(|id| (id.clone(), false)));
    }

    fn abort_call(&mut self, state: &mut State, callee_serial: u32, callee_id: ConnectionId) {
        let Some(call) = self.function_calls.get_mut(callee_serial) else {
            return;
        };

        if call.aborted {
            return;
        }

        call.aborted = true;

        if let Some(conn) = self.conns.get(&callee_id) {
            if conn.protocol_version() >= ProtocolVersion::V1_16 {
                let res = send!(
                    self,
                    conn,
                    AbortFunctionCall {
                        serial: callee_serial,
                    },
                );

                if res.is_err() {
                    state.push_remove_conn(callee_id, false);
                }
            }
        }

        if let Some(conn) = self.conns.get_mut(&call.caller_conn_id) {
            conn.remove_call(call.caller_serial);

            let res = send!(
                self,
                conn,
                CallFunctionReply {
                    serial: call.caller_serial,
                    result: CallFunctionResult::Aborted,
                },
            );

            if res.is_err() {
                state.push_remove_conn(call.caller_conn_id.clone(), false);
            }
        }
    }

    #[cfg(feature = "introspection")]
    fn remove_introspection_conn(&mut self, state: &mut State, conn_id: &ConnectionId) {
        let remove_conn = self.introspection.remove_conn(conn_id);

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_introspections = self.introspection.len();
        }

        for remove_conn in remove_conn {
            self.query_introspection
                .remove(remove_conn.serial)
                .expect("inconsistent state");

            match remove_conn.result {
                RemoveConnResult::Unavailable(pending) => {
                    for pending in pending {
                        let Some(conn) = self.conns.get(&pending.conn_id) else {
                            continue;
                        };

                        let res = send!(
                            self,
                            conn,
                            QueryIntrospectionReply {
                                serial: pending.serial,
                                result: QueryIntrospectionResult::Unavailable,
                            },
                        );

                        if res.is_err() {
                            state.push_remove_conn(pending.conn_id, false);
                        }
                    }
                }

                RemoveConnResult::Continue(type_id) => {
                    let Some(entry) = self.introspection.get_mut(type_id) else {
                        continue;
                    };

                    let serial = self.query_introspection.insert(type_id);
                    let conn_id = entry.query_random_conn(serial);
                    let conn = self.conns.get(conn_id).expect("inconsistent state");
                    let msg = QueryIntrospection { serial, type_id };

                    if send!(self, conn, msg).is_err() {
                        state.push_remove_conn(conn_id.clone(), false);
                    }
                }
            }
        }
    }
}

impl Default for Broker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct PendingFunctionCall {
    caller_serial: u32,
    caller_conn_id: ConnectionId,
    callee_obj: ObjectUuid,
    callee_svc: ServiceUuid,
    aborted: bool,
}
