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
    AddBusListenerFilter, AddChannelCapacity, BusListenerCurrentFinished, CallFunction,
    CallFunctionReply, CallFunctionResult, ChannelEndClaimed, ChannelEndClosed, ClaimChannelEnd,
    ClaimChannelEndReply, ClaimChannelEndResult, ClearBusListenerFilters, CloseChannelEnd,
    CloseChannelEndReply, CloseChannelEndResult, CreateBusListener, CreateBusListenerReply,
    CreateChannel, CreateChannelReply, CreateObject, CreateObjectReply, CreateObjectResult,
    CreateService, CreateServiceReply, CreateServiceResult, DestroyBusListener,
    DestroyBusListenerReply, DestroyBusListenerResult, DestroyObject, DestroyObjectReply,
    DestroyObjectResult, DestroyService, DestroyServiceReply, DestroyServiceResult, EmitBusEvent,
    EmitEvent, ItemReceived, Message, QueryServiceVersion, QueryServiceVersionReply,
    QueryServiceVersionResult, RemoveBusListenerFilter, SendItem, ServiceDestroyed, Shutdown,
    StartBusListener, StartBusListenerReply, StartBusListenerResult, StopBusListener,
    StopBusListenerReply, StopBusListenerResult, SubscribeEvent, SubscribeEventReply,
    SubscribeEventResult, Sync, SyncReply, UnsubscribeEvent,
};
use crate::core::{
    BusEvent, BusListenerCookie, BusListenerScope, ChannelCookie, ChannelEnd,
    ChannelEndWithCapacity, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid,
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
    ($self:expr, $conn:expr, $msg:expr) => {{
        let res = $conn.send($msg);

        #[cfg(feature = "statistics")]
        {
            $self.statistics.messages_sent += 1;
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
    svc_uuids: HashMap<ServiceCookie, (ObjectId, ServiceUuid, u32)>,
    svcs: HashMap<(ObjectUuid, ServiceUuid), Service>,
    function_calls: SerialMap<PendingFunctionCall>,
    channels: HashMap<ChannelCookie, Channel>,
    bus_listeners: HashMap<BusListenerCookie, BusListener>,
    #[cfg(feature = "statistics")]
    statistics: BrokerStatistics,
}

impl Broker {
    /// Creates a new broker.
    ///
    /// After creating a `Broker`, it must be turned into a future with [`run`](Broker::run) and
    /// polled to completion.
    pub fn new() -> Self {
        let (send, recv) = channel(FIFO_SIZE);

        Broker {
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
                    self.statistics.num_connections += 1;
                    self.statistics.connections_added += 1;
                }
            }

            ConnectionEvent::ConnectionShutdown(id) => {
                state.push_remove_conn(id, false);
            }

            ConnectionEvent::Message(id, msg) => {
                if let Err(()) = self.handle_message(state, &id, msg) {
                    state.push_remove_conn(id, false);
                }

                #[cfg(feature = "statistics")]
                {
                    self.statistics.messages_received += 1;
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
                sender.send(self.statistics.take()).ok();
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

            if let Some((conn_id, service_cookie, event)) = state.pop_unsubscribe() {
                let conn = match self.conns.get(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };
                let msg = Message::UnsubscribeEvent(UnsubscribeEvent {
                    service_cookie,
                    event,
                });
                if let Err(()) = send!(self, conn, msg) {
                    state.push_remove_conn(conn_id, false);
                }
                continue;
            }

            if let Some((conn_id, service_cookie)) = state.pop_services_destroyed() {
                let conn = match self.conns.get(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };
                let msg = Message::ServiceDestroyed(ServiceDestroyed { service_cookie });
                if let Err(()) = send!(self, conn, msg) {
                    state.push_remove_conn(conn_id, false);
                }
                continue;
            }

            if let Some((serial, conn_id, result)) = state.pop_remove_function_call() {
                let conn = match self.conns.get(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };

                let res = send!(
                    self,
                    conn,
                    Message::CallFunctionReply(CallFunctionReply { serial, result })
                );

                if res.is_err() {
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
            let _ = send!(self, conn, Message::Shutdown(Shutdown));
        }

        for bus_listener_cookie in conn.bus_listeners() {
            self.remove_bus_listener(bus_listener_cookie);
        }

        for obj_cookie in conn.objects() {
            self.remove_object(state, obj_cookie);
        }

        for (svc_cookie, event) in conn.subscriptions() {
            self.remove_subscription(state, id, svc_cookie, event);
        }

        for chann_cookie in conn.senders() {
            self.remove_channel_end(state, chann_cookie, ChannelEnd::Sender, Some(id));
        }

        for chann_cookie in conn.receivers() {
            self.remove_channel_end(state, chann_cookie, ChannelEnd::Receiver, Some(id));
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_connections -= 1;
            self.statistics.connections_shut_down += 1;
        }
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
            Message::AbortFunctionCall(_) => todo!(),

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
            | Message::ConnectReply2(_) => return Err(()),

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
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        match self.objs.entry(req.uuid) {
            Entry::Occupied(_) => send!(
                self,
                conn,
                Message::CreateObjectReply(CreateObjectReply {
                    serial: req.serial,
                    result: CreateObjectResult::DuplicateObject,
                })
            ),

            Entry::Vacant(entry) => {
                let cookie = ObjectCookie::new_v4();
                send!(
                    self,
                    conn,
                    Message::CreateObjectReply(CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::Ok(cookie),
                    })
                )?;

                let dup = self.obj_uuids.insert(cookie, req.uuid);
                debug_assert!(dup.is_none());
                entry.insert(Object::new(id.clone()));
                conn.add_object(cookie);
                state.push_create_object(ObjectId::new(req.uuid, cookie));

                #[cfg(feature = "statistics")]
                {
                    self.statistics.num_objects += 1;
                    self.statistics.objects_created += 1;
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
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let obj_uuid = match self.obj_uuids.get(&req.cookie) {
            Some(obj_uuid) => *obj_uuid,
            None => {
                return send!(
                    self,
                    conn,
                    Message::DestroyObjectReply(DestroyObjectReply {
                        serial: req.serial,
                        result: DestroyObjectResult::InvalidObject,
                    })
                );
            }
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                Message::DestroyObjectReply(DestroyObjectReply {
                    serial: req.serial,
                    result: DestroyObjectResult::ForeignObject,
                })
            );
        }

        send!(
            self,
            conn,
            Message::DestroyObjectReply(DestroyObjectReply {
                serial: req.serial,
                result: DestroyObjectResult::Ok,
            })
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
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let obj_uuid = match self.obj_uuids.get(&req.object_cookie) {
            Some(&obj_uuid) => obj_uuid,
            None => {
                return send!(
                    self,
                    conn,
                    Message::CreateServiceReply(CreateServiceReply {
                        serial: req.serial,
                        result: CreateServiceResult::InvalidObject,
                    })
                );
            }
        };

        let entry = match self.svcs.entry((obj_uuid, req.uuid)) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return send!(
                    self,
                    conn,
                    Message::CreateServiceReply(CreateServiceReply {
                        serial: req.serial,
                        result: CreateServiceResult::DuplicateService,
                    })
                );
            }
        };

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                Message::CreateServiceReply(CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::ForeignObject,
                })
            );
        }

        let svc_cookie = ServiceCookie::new_v4();
        send!(
            self,
            conn,
            Message::CreateServiceReply(CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::Ok(svc_cookie),
            })
        )?;

        let object_id = ObjectId::new(obj_uuid, req.object_cookie);
        let dup = self
            .svc_uuids
            .insert(svc_cookie, (object_id, req.uuid, req.version));
        debug_assert!(dup.is_none());
        entry.insert(Service::new());
        obj.add_service(svc_cookie);
        state.push_create_service(ServiceId::new(object_id, req.uuid, svc_cookie));

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_services += 1;
            self.statistics.services_created += 1;
        }

        Ok(())
    }

    fn destroy_service(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: DestroyService,
    ) -> Result<(), ()> {
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let obj_uuid = match self.svc_uuids.get(&req.cookie) {
            Some(ids) => ids.0.uuid,
            None => {
                return send!(
                    self,
                    conn,
                    Message::DestroyServiceReply(DestroyServiceReply {
                        serial: req.serial,
                        result: DestroyServiceResult::InvalidService,
                    })
                );
            }
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return send!(
                self,
                conn,
                Message::DestroyServiceReply(DestroyServiceReply {
                    serial: req.serial,
                    result: DestroyServiceResult::ForeignObject,
                })
            );
        }

        send!(
            self,
            conn,
            Message::DestroyServiceReply(DestroyServiceReply {
                serial: req.serial,
                result: DestroyServiceResult::Ok,
            })
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
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let (obj_uuid, svc_uuid) = match self.svc_uuids.get(&req.service_cookie) {
            Some(ids) => (ids.0.uuid, ids.1),
            None => {
                return send!(
                    self,
                    conn,
                    Message::CallFunctionReply(CallFunctionReply {
                        serial: req.serial,
                        result: CallFunctionResult::InvalidService,
                    })
                );
            }
        };

        let callee_id = match self.objs.get(&obj_uuid) {
            Some(obj) => obj.conn_id(),
            None => panic!("inconsistent state"),
        };
        let callee_conn = self.conns.get(callee_id).expect("inconsistent state");

        let serial = self.function_calls.insert(PendingFunctionCall {
            caller_serial: req.serial,
            caller_conn_id: id.clone(),
            callee_obj: obj_uuid,
            callee_svc: svc_uuid,
        });

        match self.svcs.get_mut(&(obj_uuid, svc_uuid)) {
            Some(svc) => svc.add_function_call(serial),
            None => panic!("inconsistent state"),
        }

        let res = send!(
            self,
            callee_conn,
            Message::CallFunction(CallFunction {
                serial,
                service_cookie: req.service_cookie,
                function: req.function,
                value: req.value,
            })
        );

        if res.is_err() {
            state.push_remove_conn(callee_id.clone(), false);
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_function_calls += 1;
            self.statistics.functions_called += 1;
        }

        Ok(())
    }

    fn call_function_reply(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CallFunctionReply,
    ) {
        let call = match self.function_calls.get(req.serial) {
            Some(call) => call,
            None => return,
        };

        let obj = self.objs.get(&call.callee_obj).expect("inconsistent state");
        if obj.conn_id() != id {
            return;
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_function_calls -= 1;
            self.statistics.functions_replied += 1;
        }

        let call = self.function_calls.remove(req.serial).unwrap();

        let svc = self
            .svcs
            .get_mut(&(call.callee_obj, call.callee_svc))
            .expect("inconsistent state");
        svc.remove_function_call(req.serial);

        let conn = match self.conns.get(&call.caller_conn_id) {
            Some(conn) => conn,
            None => return,
        };

        if send!(
            self,
            conn,
            Message::CallFunctionReply(CallFunctionReply {
                serial: call.caller_serial,
                result: req.result,
            })
        )
        .is_err()
        {
            state.push_remove_conn(call.caller_conn_id, false);
        }
    }

    fn subscribe_event(&mut self, id: &ConnectionId, req: SubscribeEvent) -> Result<(), ()> {
        let serial = match req.serial {
            Some(serial) => serial,
            None => return Err(()),
        };

        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let (obj_uuid, svc_uuid) = match self.svc_uuids.get(&req.service_cookie) {
            Some(ids) => (ids.0.uuid, ids.1),
            None => {
                return send!(
                    self,
                    conn,
                    Message::SubscribeEventReply(SubscribeEventReply {
                        serial,
                        result: SubscribeEventResult::InvalidService,
                    })
                );
            }
        };

        send!(
            self,
            conn,
            Message::SubscribeEventReply(SubscribeEventReply {
                serial,
                result: SubscribeEventResult::Ok,
            })
        )?;

        conn.add_subscription(req.service_cookie, req.event);
        let send_req = self
            .svcs
            .get_mut(&(obj_uuid, svc_uuid))
            .expect("inconsistent state")
            .add_subscription(req.event, id.clone());

        if send_req {
            let target_conn_id = self
                .objs
                .get_mut(&obj_uuid)
                .expect("inconsistent state")
                .conn_id();
            if let Some(target_conn) = self.conns.get(target_conn_id) {
                send!(
                    self,
                    target_conn,
                    Message::SubscribeEvent(SubscribeEvent {
                        serial: None,
                        service_cookie: req.service_cookie,
                        event: req.event,
                    })
                )
                .ok();
            }
        }

        Ok(())
    }

    fn unsubscribe_event(&mut self, state: &mut State, id: &ConnectionId, req: UnsubscribeEvent) {
        let (obj_uuid, svc_uuid) = match self.svc_uuids.get(&req.service_cookie) {
            Some(ids) => (ids.0.uuid, ids.1),
            None => return,
        };
        let svc = self
            .svcs
            .get_mut(&(obj_uuid, svc_uuid))
            .expect("inconsistent state");

        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return,
        };

        conn.remove_subscription(req.service_cookie, req.event);
        let send_unsubscribe = svc.remove_subscription(req.event, id);

        if send_unsubscribe {
            let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
            let conn_id = obj.conn_id();
            let conn = self.conns.get(conn_id).expect("inconsistent state");
            let msg = Message::UnsubscribeEvent(req);
            if let Err(()) = send!(self, conn, msg) {
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
            if conn.is_subscribed_to(req.service_cookie, req.event) {
                let msg = Message::EmitEvent(req.clone());
                if let Err(()) = send!(self, conn, msg) {
                    state.push_remove_conn(conn_id.clone(), false);
                }

                #[cfg(feature = "statistics")]
                {
                    self.statistics.events_sent += 1;
                }
            }
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.events_received += 1;
        }
    }

    fn query_service_version(
        &mut self,
        id: &ConnectionId,
        req: QueryServiceVersion,
    ) -> Result<(), ()> {
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        match self.svc_uuids.get(&req.cookie) {
            Some(&(_, _, version)) => {
                send!(
                    self,
                    conn,
                    Message::QueryServiceVersionReply(QueryServiceVersionReply {
                        serial: req.serial,
                        result: QueryServiceVersionResult::Ok(version),
                    })
                )?;
            }

            None => {
                send!(
                    self,
                    conn,
                    Message::QueryServiceVersionReply(QueryServiceVersionReply {
                        serial: req.serial,
                        result: QueryServiceVersionResult::InvalidService,
                    })
                )?;
            }
        }

        Ok(())
    }

    fn create_channel(&mut self, id: &ConnectionId, req: CreateChannel) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
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
            Message::CreateChannelReply(CreateChannelReply {
                serial: req.serial,
                cookie,
            })
        )?;

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_channels += 1;
            self.statistics.channels_created += 1;
        }

        Ok(())
    }

    fn close_channel_end(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CloseChannelEnd,
    ) -> Result<(), ()> {
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let channel = match self.channels.get(&req.cookie) {
            Some(channel) => channel,
            None => {
                send!(
                    self,
                    conn,
                    Message::CloseChannelEndReply(CloseChannelEndReply {
                        serial: req.serial,
                        result: CloseChannelEndResult::InvalidChannel,
                    })
                )?;
                return Ok(());
            }
        };

        let (result, claimed) = channel.check_close(id, req.end);
        send!(
            self,
            conn,
            Message::CloseChannelEndReply(CloseChannelEndReply {
                serial: req.serial,
                result,
            })
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
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let channel = match self.channels.get_mut(&req.cookie) {
            Some(channel) => channel,

            None => {
                send!(
                    self,
                    conn,
                    Message::ClaimChannelEndReply(ClaimChannelEndReply {
                        serial: req.serial,
                        result: ClaimChannelEndResult::InvalidChannel,
                    })
                )?;

                return Ok(());
            }
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
                    Message::ClaimChannelEndReply(ClaimChannelEndReply {
                        serial: req.serial,
                        result,
                    })
                );

                let other = self.conns.get_mut(other_id).expect("inconsistent state");

                let other_result = send!(
                    self,
                    other,
                    Message::ChannelEndClaimed(ChannelEndClaimed {
                        cookie: req.cookie,
                        end: req.end,
                    })
                );

                if other_result.is_err() {
                    state.push_remove_conn(other_id.clone(), false);
                }

                result
            }

            Err(result) => send!(
                self,
                conn,
                Message::ClaimChannelEndReply(ClaimChannelEndReply {
                    serial: req.serial,
                    result,
                })
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
            Message::AddChannelCapacity(AddChannelCapacity {
                cookie: req.cookie,
                capacity,
            })
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
                #[cfg(feature = "statistics")]
                {
                    self.statistics.items_dropped += 1;
                }

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
            Message::ItemReceived(ItemReceived {
                cookie: req.cookie,
                value: req.value,
            })
        );

        if res.is_ok() {
            #[cfg(feature = "statistics")]
            {
                self.statistics.items_sent += 1;
            }
        } else {
            #[cfg(feature = "statistics")]
            {
                self.statistics.items_dropped += 1;
            }

            state.push_remove_conn(receiver_id.clone(), false);
        }

        if let Some(add_capacity) = add_capacity {
            send!(
                self,
                sender,
                Message::AddChannelCapacity(AddChannelCapacity {
                    cookie: req.cookie,
                    capacity: add_capacity,
                })
            )
        } else {
            Ok(())
        }
    }

    fn sync(&mut self, id: &ConnectionId, req: Sync) -> Result<(), ()> {
        let conn = match self.conns.get(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        send!(
            self,
            conn,
            Message::SyncReply(SyncReply { serial: req.serial })
        )
    }

    fn create_bus_listener(&mut self, id: &ConnectionId, req: CreateBusListener) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let cookie = BusListenerCookie::new_v4();

        send!(
            self,
            conn,
            Message::CreateBusListenerReply(CreateBusListenerReply {
                serial: req.serial,
                cookie,
            })
        )?;

        #[cfg(feature = "statistics")]
        {
            self.statistics.bus_listeners_created += 1;
            self.statistics.num_bus_listeners += 1;
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
                Message::DestroyBusListenerReply(DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::InvalidBusListener,
                })
            );
        };

        if bus_listener.conn_id() == id {
            send!(
                self,
                conn,
                Message::DestroyBusListenerReply(DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::Ok,
                })
            )?;

            self.remove_bus_listener(req.cookie);
        } else {
            send!(
                self,
                conn,
                Message::DestroyBusListenerReply(DestroyBusListenerReply {
                    serial: req.serial,
                    result: DestroyBusListenerResult::InvalidBusListener,
                })
            )?;
        }

        Ok(())
    }

    fn add_bus_listener_filter(&mut self, id: &ConnectionId, req: AddBusListenerFilter) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.add_filter(req.filter);

                #[cfg(feature = "statistics")]
                {
                    self.statistics.bus_listener_filters_added += 1;
                }
            }
        }
    }

    fn remove_bus_listener_filter(&mut self, id: &ConnectionId, req: RemoveBusListenerFilter) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.remove_filter(req.filter);

                #[cfg(feature = "statistics")]
                {
                    self.statistics.bus_listener_filters_removed += 1;
                }
            }
        }
    }

    fn clear_bus_listener_filters(&mut self, id: &ConnectionId, req: ClearBusListenerFilters) {
        if let Some(bus_listener) = self.bus_listeners.get_mut(&req.cookie) {
            if bus_listener.conn_id() == id {
                bus_listener.clear_filters();

                #[cfg(feature = "statistics")]
                {
                    self.statistics.bus_listener_filters_cleared += 1;
                }
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
                Message::StartBusListenerReply(StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::InvalidBusListener,
                })
            );
        };

        if bus_listener.conn_id() != id {
            return send!(
                self,
                conn,
                Message::StartBusListenerReply(StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::InvalidBusListener,
                })
            );
        }

        if !bus_listener.start(req.scope) {
            return send!(
                self,
                conn,
                Message::StartBusListenerReply(StartBusListenerReply {
                    serial: req.serial,
                    result: StartBusListenerResult::AlreadyStarted,
                })
            );
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_bus_listeners_active += 1;
            self.statistics.bus_listeners_started += 1;
        }

        send!(
            self,
            conn,
            Message::StartBusListenerReply(StartBusListenerReply {
                serial: req.serial,
                result: StartBusListenerResult::Ok,
            })
        )?;

        if req.scope != BusListenerScope::New {
            for (&cookie, &uuid) in &self.obj_uuids {
                let object = ObjectId::new(uuid, cookie);

                if bus_listener.matches_object(object) {
                    send!(
                        self,
                        conn,
                        Message::EmitBusEvent(EmitBusEvent {
                            cookie: Some(req.cookie),
                            event: BusEvent::ObjectCreated(object),
                        })
                    )?;

                    #[cfg(feature = "statistics")]
                    {
                        self.statistics.bus_events_sent += 1;
                    }
                }
            }

            for (&service_cookie, &(object, service_uuid, _)) in &self.svc_uuids {
                let service = ServiceId::new(object, service_uuid, service_cookie);

                if bus_listener.matches_service(service) {
                    send!(
                        self,
                        conn,
                        Message::EmitBusEvent(EmitBusEvent {
                            cookie: Some(req.cookie),
                            event: BusEvent::ServiceCreated(service),
                        })
                    )?;

                    #[cfg(feature = "statistics")]
                    {
                        self.statistics.bus_events_sent += 1;
                    }
                }
            }

            send!(
                self,
                conn,
                Message::BusListenerCurrentFinished(BusListenerCurrentFinished {
                    cookie: req.cookie,
                })
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
                Message::StopBusListenerReply(StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::InvalidBusListener,
                })
            );
        };

        if bus_listener.conn_id() != id {
            return send!(
                self,
                conn,
                Message::StopBusListenerReply(StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::InvalidBusListener,
                })
            );
        }

        if bus_listener.stop() {
            #[cfg(feature = "statistics")]
            {
                self.statistics.num_bus_listeners_active -= 1;
                self.statistics.bus_listeners_stopped += 1;
            }

            send!(
                self,
                conn,
                Message::StopBusListenerReply(StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::Ok,
                })
            )
        } else {
            send!(
                self,
                conn,
                Message::StopBusListenerReply(StopBusListenerReply {
                    serial: req.serial,
                    result: StopBusListenerResult::NotStarted,
                })
            )
        }
    }

    /// Removes the object `obj_cookie` and queues up events in `state`.
    ///
    /// This function will also remove all services owned by that object as well as everything
    /// related (e.g. pending function calls). It is safe to call with an invalid `obj_cookie`.
    fn remove_object(&mut self, state: &mut State, obj_cookie: ObjectCookie) {
        let obj_uuid = match self.obj_uuids.remove(&obj_cookie) {
            Some(obj_uuid) => obj_uuid,
            None => return,
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
            self.statistics.num_objects -= 1;
            self.statistics.objects_destroyed += 1;
        }
    }

    /// Removes the service `svc_cookie` and queues up events in `state`.
    ///
    /// This function will also remove everything related to `svc_cookie`, e.g. pending function
    /// calls. It is safe to call with an invalid `svc_cookie`.
    fn remove_service(&mut self, state: &mut State, svc_cookie: ServiceCookie) {
        let (obj_id, svc_uuid, _) = match self.svc_uuids.remove(&svc_cookie) {
            Some(ids) => ids,
            None => return,
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
            state.push_remove_function_call(
                call.caller_serial,
                call.caller_conn_id,
                CallFunctionResult::InvalidService,
            );
        }

        for conn_id in svc.subscribed_conn_ids() {
            if let Some(conn) = self.conns.get_mut(conn_id) {
                conn.remove_all_subscriptions(svc_cookie);
                state.push_services_destroyed(conn_id.clone(), svc_cookie);
            }
        }

        #[cfg(feature = "statistics")]
        {
            self.statistics.num_services -= 1;
            self.statistics.services_destroyed += 1;
        }
    }

    /// Removes subscription of `event` on service `svc_cookie` made by connection `conn_id`.
    ///
    /// If this was the last subscription of `event`, a notification is sent to the owner of
    /// `svc_cookie`. It is safe to call this function with an invalid `conn_id` or `svc_cookie`.
    fn remove_subscription(
        &mut self,
        state: &mut State,
        conn_id: &ConnectionId,
        svc_cookie: ServiceCookie,
        event: u32,
    ) {
        let (obj_uuid, svc_uuid) = match self.svc_uuids.get(&svc_cookie) {
            Some(ids) => (ids.0.uuid, ids.1),
            None => return,
        };

        // The connection might already have been removed.
        if let Some(conn) = self.conns.get_mut(conn_id) {
            conn.remove_subscription(svc_cookie, event);
        }

        let svc = self
            .svcs
            .get_mut(&(obj_uuid, svc_uuid))
            .expect("inconsistent state");
        if svc.remove_subscription(event, conn_id) {
            let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
            state.push_unsubscribe(obj.conn_id().clone(), svc_cookie, event);
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
        let mut channel = match self.channels.entry(cookie) {
            Entry::Occupied(channel) => channel,
            Entry::Vacant(_) => return,
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
                    let res = send!(
                        self,
                        other,
                        Message::ChannelEndClosed(ChannelEndClosed { cookie, end })
                    );

                    if res.is_err() {
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
                self.statistics.num_channels -= 1;
                self.statistics.channels_closed += 1;
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
            self.statistics.bus_listeners_destroyed += 1;
            self.statistics.num_bus_listeners -= 1;

            if bus_listener.is_started() {
                self.statistics.num_bus_listeners_active -= 1;
                self.statistics.bus_listeners_stopped += 1;
            }
        }
    }

    fn emit_bus_event(&mut self, state: &mut State, event: BusEvent) {
        let mut dups = HashSet::new();
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
                Message::EmitBusEvent(EmitBusEvent {
                    cookie: None,
                    event,
                })
            );

            if res.is_ok() {
                #[cfg(feature = "statistics")]
                {
                    self.statistics.bus_events_sent += 1;
                }
            } else {
                remove_conns.insert(conn_id);
            }
        }

        state.push_remove_conns(remove_conns.into_iter().map(|id| (id.clone(), false)));
    }
}

impl Default for Broker {
    fn default() -> Self {
        Broker::new()
    }
}

#[derive(Debug)]
struct PendingFunctionCall {
    caller_serial: u32,
    caller_conn_id: ConnectionId,
    callee_obj: ObjectUuid,
    callee_svc: ServiceUuid,
}
