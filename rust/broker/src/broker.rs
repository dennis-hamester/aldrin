mod conn_state;
mod error;
mod handle;
mod object;
mod service;
mod state;

use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionId;
use crate::serial_map::SerialMap;
use aldrin_proto::*;
use conn_state::ConnectionState;
use futures_channel::mpsc::{unbounded, UnboundedReceiver};
use futures_core::stream::FusedStream;
use futures_util::stream::StreamExt;
use object::{Object, ObjectCookie, ObjectUuid};
use service::{Service, ServiceCookie, ServiceUuid};
use state::State;
use std::collections::hash_map::{Entry, HashMap};
use std::num::NonZeroUsize;
use uuid::Uuid;

pub use error::BrokerError;
pub use handle::BrokerHandle;

const DEFAULT_FIFO_SIZE: usize = 32;

#[derive(Debug)]
pub struct Broker {
    fifo_size: Option<NonZeroUsize>,
    recv: UnboundedReceiver<ConnectionEvent>,
    handle: Option<BrokerHandle>,
    conns: HashMap<ConnectionId, ConnectionState>,
    obj_uuids: HashMap<ObjectCookie, ObjectUuid>,
    objs: HashMap<ObjectUuid, Object>,
    svc_uuids: HashMap<ServiceCookie, (ObjectUuid, ObjectCookie, ServiceUuid)>,
    svcs: HashMap<(ObjectUuid, ServiceUuid), Service>,

    /// Map of pending function calls.
    ///
    /// The serial of an entry refers to the callee and will be referenced in
    /// [`CallFunctionReply`] message.
    ///
    /// The elements of the tuple are:
    /// 1. Serial used by the caller in the [`CallFunction`] message.
    /// 2. [`ConnectionId`] of the caller.
    /// 3. [`ObjectUuid`] which owns the called service.
    /// 4. [`ServiceUuid`] on which the function has been called.
    function_calls: SerialMap<(u32, ConnectionId, ObjectUuid, ServiceUuid)>,
}

impl Broker {
    /// Creates a broker with the default fifo size.
    ///
    /// The default fifo size is 32. Use [`Broker::with_fifo_size`] to create a broker with a custom
    /// fifo size.
    pub fn new() -> Self {
        Self::with_fifo_size(NonZeroUsize::new(DEFAULT_FIFO_SIZE))
    }

    /// Creates a broker with a custom fifo size.
    ///
    /// If `fifo_size` is `None`, then the internal fifo will be unbounded, which should be used
    /// with care. If the fifo overflows, [`Broker::run`] will return immediately with an error.
    pub fn with_fifo_size(fifo_size: Option<NonZeroUsize>) -> Self {
        let (send, recv) = unbounded();

        Broker {
            fifo_size,
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

    pub fn handle(&self) -> &BrokerHandle {
        self.handle.as_ref().unwrap()
    }

    pub async fn run(mut self) -> Result<(), BrokerError> {
        self.handle.take().unwrap();

        let mut state = State::new();
        let mut queue = Vec::with_capacity(self.fifo_size.map(NonZeroUsize::get).unwrap_or(1));

        loop {
            if state.shutdown_now()
                || (state.shutdown_idle() && self.conns.is_empty())
                || self.recv.is_terminated()
            {
                return Ok(());
            }

            match self.recv.next().await {
                Some(ev) => queue.push(ev),
                None => return Ok(()),
            };

            if let Some(fifo_size) = self.fifo_size {
                while let Ok(Some(ev)) = self.recv.try_next() {
                    if queue.len() >= fifo_size.get() {
                        return Err(BrokerError::FifoOverflow);
                    } else {
                        queue.push(ev);
                    }
                }
            }

            for ev in queue.drain(..) {
                self.handle_event(&mut state, ev);
                self.process_loop_result(&mut state);
            }
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
                        .filter(|(_, c)| c.services_created_subscribed()),
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
                        .filter(|(_, c)| c.services_destroyed_subscribed()),
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
        conn.send(Message::Shutdown).ok();

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
            Message::CreateObject(req) => self.create_object(state, id, req),
            Message::DestroyObject(req) => self.destroy_object(state, id, req),
            Message::SubscribeObjects(req) => self.subscribe_objects(id, req),
            Message::UnsubscribeObjects => self.unsubscribe_objects(id),
            Message::CreateService(req) => self.create_service(state, id, req),
            Message::DestroyService(req) => self.destroy_service(state, id, req),
            Message::SubscribeServicesCreated(req) => self.subscribe_services_created(id, req),
            Message::UnsubscribeServicesCreated => self.unsubscribe_services_created(id),
            Message::SubscribeServicesDestroyed => self.subscribe_services_destroyed(id),
            Message::UnsubscribeServicesDestroyed => self.unsubscribe_services_destroyed(id),
            Message::CallFunction(req) => self.call_function(state, id, req),
            Message::CallFunctionReply(req) => self.call_function_reply(state, req),
            Message::SubscribeEvent(req) => self.subscribe_event(id, req),
            Message::UnsubscribeEvent(req) => self.unsubscribe_event(state, id, req),
            Message::EmitEvent(req) => self.emit_event(state, req),

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObjectReply(_)
            | Message::DestroyObjectReply(_)
            | Message::SubscribeObjectsReply(_)
            | Message::ObjectCreatedEvent(_)
            | Message::ObjectDestroyedEvent(_)
            | Message::CreateServiceReply(_)
            | Message::SubscribeServicesCreatedReply(_)
            | Message::ServiceCreatedEvent(_)
            | Message::DestroyServiceReply(_)
            | Message::ServiceDestroyedEvent(_)
            | Message::SubscribeEventReply(_) => Err(()),

            Message::Shutdown => unreachable!(), // Handled by connection.
        }
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
                let cookie = ObjectCookie(Uuid::new_v4());
                conn.send(Message::CreateObjectReply(CreateObjectReply {
                    serial: req.serial,
                    result: CreateObjectResult::Ok(cookie.0),
                }))?;
                let dup = self.obj_uuids.insert(cookie, uuid);
                debug_assert!(dup.is_none());
                entry.insert(Object::new(id.clone()));
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

    fn unsubscribe_objects(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_subscribed(false);
        Ok(())
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

        let svc_cookie = ServiceCookie(Uuid::new_v4());
        conn.send(Message::CreateServiceReply(CreateServiceReply {
            serial: req.serial,
            result: CreateServiceResult::Ok(svc_cookie.0),
        }))?;

        let dup = self
            .svc_uuids
            .insert(svc_cookie, (obj_uuid, obj_cookie, svc_uuid));
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
        let (obj_uuid, _, _) = match self.svc_uuids.get(&svc_cookie) {
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

    fn subscribe_services_created(
        &mut self,
        id: &ConnectionId,
        req: SubscribeServicesCreated,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_created_subscribed(true);

        if let Some(serial) = req.serial {
            for (&svc_cookie, &(obj_uuid, obj_cookie, svc_uuid)) in &self.svc_uuids {
                conn.send(Message::ServiceCreatedEvent(ServiceCreatedEvent {
                    object_uuid: obj_uuid.0,
                    object_cookie: obj_cookie.0,
                    uuid: svc_uuid.0,
                    cookie: svc_cookie.0,
                    serial: Some(serial),
                }))?;
            }

            conn.send(Message::SubscribeServicesCreatedReply(
                SubscribeServicesCreatedReply { serial },
            ))?;
        }

        Ok(())
    }

    fn unsubscribe_services_created(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_created_subscribed(false);
        Ok(())
    }

    fn subscribe_services_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_destroyed_subscribed(true);
        Ok(())
    }

    fn unsubscribe_services_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_destroyed_subscribed(false);
        Ok(())
    }

    fn call_function(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: CallFunction,
    ) -> Result<(), ()> {
        let svc_cookie = ServiceCookie(req.service_cookie);
        let (obj_uuid, _, svc_uuid) = match self.svc_uuids.get(&svc_cookie) {
            Some(uuids) => *uuids,
            None => {
                let conn = match self.conns.get_mut(id) {
                    Some(conn) => conn,
                    None => return Ok(()),
                };

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

        let serial = self
            .function_calls
            .insert((req.serial, id.clone(), obj_uuid, svc_uuid));
        let res = callee_conn.send(Message::CallFunction(CallFunction {
            serial,
            service_cookie: svc_cookie.0,
            function: req.function,
            args: req.args,
        }));

        if res.is_ok() {
            let svc = match self.svcs.get_mut(&(obj_uuid, svc_uuid)) {
                Some(svc) => svc,
                None => panic!("inconsistent state"),
            };

            svc.add_function_call(serial);
        } else {
            self.function_calls.remove(serial);
            state.push_remove_conn(callee_id.clone());
        }

        Ok(())
    }

    fn call_function_reply(&mut self, state: &mut State, req: CallFunctionReply) -> Result<(), ()> {
        let (serial, conn_id, obj_uuid, svc_uuid) = match self.function_calls.remove(req.serial) {
            Some(caller) => caller,
            None => return Ok(()),
        };

        let svc = self
            .svcs
            .get_mut(&(obj_uuid, svc_uuid))
            .expect("inconsistent state");
        svc.remove_function_call(req.serial);

        let res = self
            .conns
            .get_mut(&conn_id)
            .expect("inconsistent state")
            .send(Message::CallFunctionReply(CallFunctionReply {
                serial,
                result: req.result,
            }));
        if res.is_err() {
            state.push_remove_conn(conn_id);
        }

        Ok(())
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
        let (obj_uuid, _, svc_uuid) = match self.svc_uuids.get(&svc_cookie) {
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

    fn unsubscribe_event(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        req: UnsubscribeEvent,
    ) -> Result<(), ()> {
        let svc_cookie = ServiceCookie(req.service_cookie);
        let (obj_uuid, _, svc_uuid) = match self.svc_uuids.get(&svc_cookie) {
            Some(ids) => *ids,
            None => return Ok(()),
        };
        let svc = self
            .svcs
            .get_mut(&(obj_uuid, svc_uuid))
            .expect("inconsistent state");

        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
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

        Ok(())
    }

    fn emit_event(&mut self, state: &mut State, req: EmitEvent) -> Result<(), ()> {
        let svc_cookie = ServiceCookie(req.service_cookie);
        for (conn_id, conn) in self.conns.iter_mut() {
            if conn.is_subscribed_to(svc_cookie, req.event) {
                let msg = Message::EmitEvent(req.clone());
                if let Err(()) = conn.send(msg) {
                    state.push_remove_conn(conn_id.clone());
                }
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
        let (obj_uuid, obj_cookie, svc_uuid) = match self.svc_uuids.remove(&svc_cookie) {
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
            let (serial, conn_id, _, _) = self
                .function_calls
                .remove(serial)
                .expect("inconsistent state");
            state.push_remove_function_call(serial, conn_id, CallFunctionResult::InvalidService);
        }

        for conn_id in svc.subscribed_conn_ids() {
            if let Some(conn) = self.conns.get_mut(conn_id) {
                conn.remove_all_subscriptions(svc_cookie);
                if !conn.services_destroyed_subscribed() {
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
        let (obj_uuid, _, svc_uuid) = match self.svc_uuids.get(&svc_cookie) {
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
    // This should really be IntoIterator, but we're hitting a compiler bug:
    // https://github.com/tokio-rs/tokio/issues/1835
    // https://github.com/rust-lang/rust/issues/60658
    I: Iterator<Item = (&'a ConnectionId, &'a mut ConnectionState)>,
{
    for (id, conn) in conns {
        // Shutdown connection on error, but don't abort loop.
        if let Err(()) = conn.send(msg.clone()) {
            state.push_remove_conn(id.clone());
        }
    }
}
