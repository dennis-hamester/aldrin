mod conn_state;
mod error;
mod handle;
mod object;
mod service;
mod state;
#[cfg(test)]
mod test;

use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionId;
use crate::serial_map::SerialMap;
use aldrin_proto::{
    CallFunction, CallFunctionReply, CallFunctionResult, CreateObject, CreateObjectReply,
    CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult, DestroyObject,
    DestroyObjectReply, DestroyObjectResult, DestroyService, DestroyServiceReply,
    DestroyServiceResult, EmitEvent, Message, ObjectCreatedEvent, ObjectDestroyedEvent,
    QueryObject, QueryObjectReply, QueryObjectResult, QueryServiceVersion,
    QueryServiceVersionReply, QueryServiceVersionResult, ServiceCreatedEvent,
    ServiceDestroyedEvent, SubscribeEvent, SubscribeEventReply, SubscribeEventResult,
    SubscribeObjects, SubscribeObjectsReply, SubscribeServices, SubscribeServicesReply,
    UnsubscribeEvent,
};
use conn_state::ConnectionState;
use futures_channel::mpsc::{channel, Receiver};
use futures_util::stream::StreamExt;
use object::{Object, ObjectCookie, ObjectUuid};
use service::{Service, ServiceCookie, ServiceUuid};
use state::State;
use std::collections::hash_map::{Entry, HashMap};

pub use error::BrokerShutdown;
pub use handle::BrokerHandle;

const FIFO_SIZE: usize = 32;

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
    svc_uuids: HashMap<ServiceCookie, (ObjectUuid, ObjectCookie, ServiceUuid, u32)>,
    svcs: HashMap<(ObjectUuid, ServiceUuid), Service>,
    function_calls: SerialMap<PendingFunctionCall>,
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
                return;
            }

            let ev = match self.recv.next().await {
                Some(ev) => ev,
                None => return,
            };

            self.handle_event(&mut state, ev);
            self.process_loop_result(&mut state);
        }
    }

    fn handle_event(&mut self, state: &mut State, ev: ConnectionEvent) {
        match ev {
            ConnectionEvent::NewConnection(id, sender) => {
                let dup = self.conns.insert(id, ConnectionState::new(sender));
                debug_assert!(dup.is_none());
            }

            ConnectionEvent::ConnectionShutdown(id) => {
                state.push_remove_conn(id);
            }

            ConnectionEvent::Message(id, msg) => {
                if let Err(()) = self.handle_message(state, &id, msg) {
                    state.push_remove_conn(id);
                }
            }

            ConnectionEvent::ShutdownBroker => {
                state.push_remove_conns(self.conns.keys().cloned());
                state.set_shutdown_now();
            }

            ConnectionEvent::ShutdownIdleBroker => {
                state.set_shutdown_idle();
            }

            ConnectionEvent::ShutdownConnection(id) => {
                state.push_remove_conn(id);
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

            if let Some(conn) = state.pop_remove_conn() {
                self.shutdown_connection(state, &conn);
                continue;
            }

            if let Some((uuid, cookie)) = state.pop_add_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_subscribed()),
                    state,
                    Message::ObjectCreatedEvent(ObjectCreatedEvent {
                        uuid: uuid.0,
                        cookie: cookie.0,
                        serial: None,
                    }),
                );
                continue;
            }

            if let Some((obj_uuid, obj_cookie, uuid, cookie)) = state.pop_add_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_subscribed()),
                    state,
                    Message::ServiceCreatedEvent(ServiceCreatedEvent {
                        object_uuid: obj_uuid.0,
                        object_cookie: obj_cookie.0,
                        uuid: uuid.0,
                        cookie: cookie.0,
                        serial: None,
                    }),
                );
                continue;
            }

            if let Some((conn_id, svc_cookie, event)) = state.pop_unsubscribe() {
                let conn = match self.conns.get_mut(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };
                let msg = Message::UnsubscribeEvent(UnsubscribeEvent {
                    service_cookie: svc_cookie.0,
                    event,
                });
                if let Err(()) = conn.send(msg) {
                    state.push_remove_conn(conn_id);
                }
                continue;
            }

            if let Some((obj_uuid, obj_cookie, uuid, cookie)) = state.pop_remove_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_subscribed()),
                    state,
                    Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
                        object_uuid: obj_uuid.0,
                        object_cookie: obj_cookie.0,
                        uuid: uuid.0,
                        cookie: cookie.0,
                    }),
                );
                continue;
            }

            if let Some((conn_id, obj_uuid, obj_cookie, svc_uuid, svc_cookie)) =
                state.pop_remove_subscriptions()
            {
                let conn = match self.conns.get_mut(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };
                let msg = Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
                    object_uuid: obj_uuid.0,
                    object_cookie: obj_cookie.0,
                    uuid: svc_uuid.0,
                    cookie: svc_cookie.0,
                });
                if let Err(()) = conn.send(msg) {
                    state.push_remove_conn(conn_id);
                }
                continue;
            }

            if let Some((uuid, cookie)) = state.pop_remove_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_subscribed()),
                    state,
                    Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
                        uuid: uuid.0,
                        cookie: cookie.0,
                    }),
                );
                continue;
            }

            if let Some((serial, conn_id, result)) = state.pop_remove_function_call() {
                let conn = match self.conns.get_mut(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };

                let res = conn.send(Message::CallFunctionReply(CallFunctionReply {
                    serial,
                    result,
                }));

                if res.is_err() {
                    state.push_remove_conn(conn_id);
                }
                continue;
            }

            debug_assert!(!state.has_work_left());
            break;
        }
    }

    fn shutdown_connection(&mut self, state: &mut State, id: &ConnectionId) {
        let mut conn = match self.conns.remove(id) {
            Some(conn) => conn,
            None => return,
        };

        // Ignore errors here
        conn.send(Message::Shutdown(())).ok();

        for obj_cookie in conn.objects() {
            self.remove_object(state, obj_cookie);
        }

        for (svc_cookie, event) in conn.subscriptions() {
            self.remove_subscription(state, id, svc_cookie, event);
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
            Message::SubscribeObjects(req) => self.subscribe_objects(id, req)?,
            Message::UnsubscribeObjects(()) => self.unsubscribe_objects(id),
            Message::CreateService(req) => self.create_service(state, id, req)?,
            Message::DestroyService(req) => self.destroy_service(state, id, req)?,
            Message::SubscribeServices(req) => self.subscribe_services(id, req)?,
            Message::UnsubscribeServices(()) => self.unsubscribe_services(id),
            Message::CallFunction(req) => self.call_function(state, id, req)?,
            Message::CallFunctionReply(req) => self.call_function_reply(state, req),
            Message::SubscribeEvent(req) => self.subscribe_event(id, req)?,
            Message::UnsubscribeEvent(req) => self.unsubscribe_event(state, id, req),
            Message::EmitEvent(req) => self.emit_event(state, req),
            Message::QueryObject(req) => self.query_object(id, req)?,
            Message::QueryServiceVersion(req) => self.query_service_version(id, req)?,

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObjectReply(_)
            | Message::DestroyObjectReply(_)
            | Message::SubscribeObjectsReply(_)
            | Message::ObjectCreatedEvent(_)
            | Message::ObjectDestroyedEvent(_)
            | Message::CreateServiceReply(_)
            | Message::DestroyServiceReply(_)
            | Message::SubscribeServicesReply(_)
            | Message::ServiceCreatedEvent(_)
            | Message::ServiceDestroyedEvent(_)
            | Message::SubscribeEventReply(_)
            | Message::QueryObjectReply(_)
            | Message::QueryServiceVersionReply(_) => return Err(()),

            Message::Shutdown(()) => unreachable!(), // Handled by connection.
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

        let uuid = ObjectUuid(req.uuid);
        match self.objs.entry(uuid) {
            Entry::Occupied(_) => conn.send(Message::CreateObjectReply(CreateObjectReply {
                serial: req.serial,
                result: CreateObjectResult::DuplicateObject,
            })),

            Entry::Vacant(entry) => {
                let cookie = ObjectCookie::new_v4();
                conn.send(Message::CreateObjectReply(CreateObjectReply {
                    serial: req.serial,
                    result: CreateObjectResult::Ok(cookie.0),
                }))?;
                let dup = self.obj_uuids.insert(cookie, uuid);
                debug_assert!(dup.is_none());
                entry.insert(Object::new(id.clone(), cookie));
                conn.add_object(cookie);
                state.push_add_obj(uuid, cookie);
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
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let obj_cookie = ObjectCookie(req.cookie);
        let obj_uuid = match self.obj_uuids.get(&obj_cookie) {
            Some(obj_uuid) => *obj_uuid,
            None => {
                return conn.send(Message::DestroyObjectReply(DestroyObjectReply {
                    serial: req.serial,
                    result: DestroyObjectResult::InvalidObject,
                }));
            }
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return conn.send(Message::DestroyObjectReply(DestroyObjectReply {
                serial: req.serial,
                result: DestroyObjectResult::ForeignObject,
            }));
        }

        conn.send(Message::DestroyObjectReply(DestroyObjectReply {
            serial: req.serial,
            result: DestroyObjectResult::Ok,
        }))?;

        self.remove_object(state, obj_cookie);
        Ok(())
    }

    fn subscribe_objects(&mut self, id: &ConnectionId, req: SubscribeObjects) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_subscribed(true);

        if let Some(serial) = req.serial {
            for (&cookie, &uuid) in &self.obj_uuids {
                conn.send(Message::ObjectCreatedEvent(ObjectCreatedEvent {
                    uuid: uuid.0,
                    cookie: cookie.0,
                    serial: Some(serial),
                }))?;
            }

            conn.send(Message::SubscribeObjectsReply(SubscribeObjectsReply {
                serial,
            }))?;
        }

        Ok(())
    }

    fn unsubscribe_objects(&mut self, id: &ConnectionId) {
        if let Some(conn) = self.conns.get_mut(id) {
            conn.set_objects_subscribed(false);
        }
    }

    fn create_service(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CreateService,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let obj_cookie = ObjectCookie(req.object_cookie);
        let obj_uuid = match self.obj_uuids.get(&obj_cookie) {
            Some(&obj_uuid) => obj_uuid,
            None => {
                return conn.send(Message::CreateServiceReply(CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::InvalidObject,
                }));
            }
        };

        let svc_uuid = ServiceUuid(req.uuid);
        let entry = match self.svcs.entry((obj_uuid, svc_uuid)) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return conn.send(Message::CreateServiceReply(CreateServiceReply {
                    serial: req.serial,
                    result: CreateServiceResult::DuplicateService,
                }));
            }
        };

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return conn.send(Message::CreateServiceReply(CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::ForeignObject,
            }));
        }

        let svc_cookie = ServiceCookie::new_v4();
        conn.send(Message::CreateServiceReply(CreateServiceReply {
            serial: req.serial,
            result: CreateServiceResult::Ok(svc_cookie.0),
        }))?;

        let dup = self
            .svc_uuids
            .insert(svc_cookie, (obj_uuid, obj_cookie, svc_uuid, req.version));
        debug_assert!(dup.is_none());
        entry.insert(Service::new());
        obj.add_service(svc_cookie);
        state.push_add_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);

        Ok(())
    }

    fn destroy_service(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: DestroyService,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let svc_cookie = ServiceCookie(req.cookie);
        let (obj_uuid, _, _, _) = match self.svc_uuids.get(&svc_cookie) {
            Some(ids) => *ids,
            None => {
                return conn.send(Message::DestroyServiceReply(DestroyServiceReply {
                    serial: req.serial,
                    result: DestroyServiceResult::InvalidService,
                }));
            }
        };

        let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return conn.send(Message::DestroyServiceReply(DestroyServiceReply {
                serial: req.serial,
                result: DestroyServiceResult::ForeignObject,
            }));
        }

        conn.send(Message::DestroyServiceReply(DestroyServiceReply {
            serial: req.serial,
            result: DestroyServiceResult::Ok,
        }))?;

        self.remove_service(state, svc_cookie);
        Ok(())
    }

    fn subscribe_services(&mut self, id: &ConnectionId, req: SubscribeServices) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_subscribed(true);

        if let Some(serial) = req.serial {
            for (&svc_cookie, &(obj_uuid, obj_cookie, svc_uuid, _)) in &self.svc_uuids {
                conn.send(Message::ServiceCreatedEvent(ServiceCreatedEvent {
                    object_uuid: obj_uuid.0,
                    object_cookie: obj_cookie.0,
                    uuid: svc_uuid.0,
                    cookie: svc_cookie.0,
                    serial: Some(serial),
                }))?;
            }

            conn.send(Message::SubscribeServicesReply(SubscribeServicesReply {
                serial,
            }))?;
        }

        Ok(())
    }

    fn unsubscribe_services(&mut self, id: &ConnectionId) {
        if let Some(conn) = self.conns.get_mut(id) {
            conn.set_services_subscribed(false);
        }
    }

    fn call_function(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CallFunction,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let svc_cookie = ServiceCookie(req.service_cookie);
        let (obj_uuid, _, svc_uuid, _) = match self.svc_uuids.get(&svc_cookie) {
            Some(uuids) => *uuids,
            None => {
                return conn.send(Message::CallFunctionReply(CallFunctionReply {
                    serial: req.serial,
                    result: CallFunctionResult::InvalidService,
                }));
            }
        };

        let callee_id = match self.objs.get(&obj_uuid) {
            Some(obj) => obj.conn_id(),
            None => panic!("inconsistent state"),
        };
        let callee_conn = self.conns.get_mut(callee_id).expect("inconsistent state");

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

        let res = callee_conn.send(Message::CallFunction(CallFunction {
            serial,
            service_cookie: svc_cookie.0,
            function: req.function,
            args: req.args,
        }));

        if res.is_err() {
            state.push_remove_conn(callee_id.clone());
        }

        Ok(())
    }

    fn call_function_reply(&mut self, state: &mut State, req: CallFunctionReply) {
        let call = match self.function_calls.remove(req.serial) {
            Some(call) => call,
            None => return,
        };

        let svc = self
            .svcs
            .get_mut(&(call.callee_obj, call.callee_svc))
            .expect("inconsistent state");
        svc.remove_function_call(req.serial);

        let conn = match self.conns.get_mut(&call.caller_conn_id) {
            Some(conn) => conn,
            None => return,
        };

        if conn
            .send(Message::CallFunctionReply(CallFunctionReply {
                serial: call.caller_serial,
                result: req.result,
            }))
            .is_err()
        {
            state.push_remove_conn(call.caller_conn_id);
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

        let svc_cookie = ServiceCookie(req.service_cookie);
        let (obj_uuid, _, svc_uuid, _) = match self.svc_uuids.get(&svc_cookie) {
            Some(&ids) => ids,
            None => {
                return conn.send(Message::SubscribeEventReply(SubscribeEventReply {
                    serial,
                    result: SubscribeEventResult::InvalidService,
                }));
            }
        };

        conn.send(Message::SubscribeEventReply(SubscribeEventReply {
            serial,
            result: SubscribeEventResult::Ok,
        }))?;

        conn.add_subscription(svc_cookie, req.event);
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
            if let Some(target_conn) = self.conns.get_mut(target_conn_id) {
                target_conn
                    .send(Message::SubscribeEvent(SubscribeEvent {
                        serial: None,
                        service_cookie: req.service_cookie,
                        event: req.event,
                    }))
                    .ok();
            }
        }

        Ok(())
    }

    fn unsubscribe_event(&mut self, state: &mut State, id: &ConnectionId, req: UnsubscribeEvent) {
        let svc_cookie = ServiceCookie(req.service_cookie);
        let (obj_uuid, _, svc_uuid, _) = match self.svc_uuids.get(&svc_cookie) {
            Some(ids) => *ids,
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

        conn.remove_subscription(svc_cookie, req.event);
        let send_unsubscribe = svc.remove_subscription(req.event, id);

        if send_unsubscribe {
            let obj = self.objs.get(&obj_uuid).expect("inconsistent state");
            let conn_id = obj.conn_id();
            let conn = self.conns.get_mut(conn_id).expect("inconsistent state");
            let msg = Message::UnsubscribeEvent(req);
            if let Err(()) = conn.send(msg) {
                state.push_remove_conn(conn_id.clone());
            }
        }
    }

    fn emit_event(&mut self, state: &mut State, req: EmitEvent) {
        let svc_cookie = ServiceCookie(req.service_cookie);
        for (conn_id, conn) in self.conns.iter_mut() {
            if conn.is_subscribed_to(svc_cookie, req.event) {
                let msg = Message::EmitEvent(req.clone());
                if let Err(()) = conn.send(msg) {
                    state.push_remove_conn(conn_id.clone());
                }
            }
        }
    }

    fn query_object(&mut self, id: &ConnectionId, req: QueryObject) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        let uuid = ObjectUuid(req.uuid);
        let obj = match self.objs.get(&uuid) {
            Some(obj) => obj,
            None => {
                return conn.send(Message::QueryObjectReply(QueryObjectReply {
                    serial: req.serial,
                    result: QueryObjectResult::InvalidObject,
                }));
            }
        };

        conn.send(Message::QueryObjectReply(QueryObjectReply {
            serial: req.serial,
            result: QueryObjectResult::Cookie(obj.cookie().0),
        }))?;

        if !req.with_services {
            return Ok(());
        }

        for cookie in obj.services() {
            let (_, _, uuid, _) = self.svc_uuids.get(&cookie).expect("inconsistent state");
            conn.send(Message::QueryObjectReply(QueryObjectReply {
                serial: req.serial,
                result: QueryObjectResult::Service {
                    uuid: uuid.0,
                    cookie: cookie.0,
                },
            }))?;
        }

        conn.send(Message::QueryObjectReply(QueryObjectReply {
            serial: req.serial,
            result: QueryObjectResult::Done,
        }))?;

        Ok(())
    }

    fn query_service_version(
        &mut self,
        id: &ConnectionId,
        req: QueryServiceVersion,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        match self.svc_uuids.get(&ServiceCookie(req.cookie)) {
            Some(&(_, _, _, version)) => {
                conn.send(Message::QueryServiceVersionReply(
                    QueryServiceVersionReply {
                        serial: req.serial,
                        result: QueryServiceVersionResult::Ok(version),
                    },
                ))?;
            }

            None => {
                conn.send(Message::QueryServiceVersionReply(
                    QueryServiceVersionReply {
                        serial: req.serial,
                        result: QueryServiceVersionResult::InvalidService,
                    },
                ))?;
            }
        }

        Ok(())
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
        state.push_remove_obj(obj_uuid, obj_cookie);

        // The connection might already have been removed. E.g. when this function is called by
        // `shutdown_connection`.
        if let Some(conn) = self.conns.get_mut(obj.conn_id()) {
            conn.remove_object(obj_cookie);
        }

        for svc_cookie in obj.services() {
            self.remove_service(state, svc_cookie);
        }
    }

    /// Removes the service `svc_cookie` and queues up events in `state`.
    ///
    /// This function will also remove everything related to `svc_cookie`, e.g. pending function
    /// calls. It is safe to call with an invalid `svc_cookie`.
    fn remove_service(&mut self, state: &mut State, svc_cookie: ServiceCookie) {
        let (obj_uuid, obj_cookie, svc_uuid, _) = match self.svc_uuids.remove(&svc_cookie) {
            Some(ids) => ids,
            None => return,
        };

        let svc = self
            .svcs
            .remove(&(obj_uuid, svc_uuid))
            .expect("inconsistent state");
        state.push_remove_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);

        // The object might already have been removed.
        if let Some(obj) = self.objs.get_mut(&obj_uuid) {
            obj.remove_service(svc_cookie);
        }

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
                if !conn.services_subscribed() {
                    state.push_remove_subscriptions(
                        conn_id.clone(),
                        obj_uuid,
                        obj_cookie,
                        svc_uuid,
                        svc_cookie,
                    );
                }
            }
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
        let (obj_uuid, _, svc_uuid, _) = match self.svc_uuids.get(&svc_cookie) {
            Some(ids) => *ids,
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
}

impl Default for Broker {
    fn default() -> Self {
        Broker::new()
    }
}

fn broadcast<'a, I>(conns: I, state: &mut State, msg: Message)
where
    I: IntoIterator<Item = (&'a ConnectionId, &'a mut ConnectionState)>,
{
    for (id, conn) in conns {
        // Shutdown connection on error, but don't abort loop.
        if let Err(()) = conn.send(msg.clone()) {
            state.push_remove_conn(id.clone());
        }
    }
}

#[derive(Debug)]
struct PendingFunctionCall {
    caller_serial: u32,
    caller_conn_id: ConnectionId,
    callee_obj: ObjectUuid,
    callee_svc: ServiceUuid,
}
