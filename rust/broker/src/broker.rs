// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod builder;
mod conn_state;
mod error;
mod event;
mod handle;
mod object;
mod service;
mod state;

use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionId;
use crate::serial_map::SerialMap;
use aldrin_proto::*;
use conn_state::ConnectionState;
use futures_channel::mpsc::{channel, Receiver};
use futures_util::stream::StreamExt;
use object::{Object, ObjectCookie, ObjectUuid};
use service::{Service, ServiceCookie, ServiceUuid};
use state::State;
use std::collections::hash_map::{Entry, HashMap};
use uuid::Uuid;

pub use builder::BrokerBuilder;
pub use error::BrokerError;
pub use handle::BrokerHandle;

pub(crate) use event::BrokerEvent;

#[derive(Debug)]
pub struct Broker {
    recv: Receiver<ConnectionEvent>,
    handle: Option<BrokerHandle>,
    conns: HashMap<ConnectionId, ConnectionState>,
    obj_uuids: HashMap<ObjectCookie, ObjectUuid>,
    objs: HashMap<ObjectUuid, Object>,
    svc_uuids: HashMap<ServiceCookie, (ObjectUuid, ObjectCookie, ServiceUuid)>,
    svcs: HashMap<(ObjectUuid, ServiceUuid), Service>,
    function_calls: SerialMap<(u32, ConnectionId)>,
}

impl Broker {
    pub fn builder() -> BrokerBuilder {
        BrokerBuilder::new()
    }

    pub(crate) fn new(fifo_size: usize) -> Self {
        let (send, recv) = channel(fifo_size);

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

    pub fn handle(&self) -> &BrokerHandle {
        self.handle.as_ref().unwrap()
    }

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

            self.handle_event(&mut state, ev).await;
            self.process_loop_result(&mut state).await;
        }
    }

    async fn handle_event(&mut self, state: &mut State, ev: ConnectionEvent) {
        match ev {
            ConnectionEvent::NewConnection(id, sender) => {
                let dup = self.conns.insert(id, ConnectionState::new(sender));
                debug_assert!(dup.is_none());
            }

            ConnectionEvent::ConnectionShutdown(id) => {
                state.push_remove_conn(id);
            }

            ConnectionEvent::Message(id, msg) => {
                if let Err(()) = self.handle_message(state, &id, msg).await {
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

    async fn process_loop_result(&mut self, state: &mut State) {
        loop {
            // The order in which events are processed and sent to clients matters here.
            // Always remove connections first. That way we never actually try to send events to
            // clients, which are known to be shut down.
            // Then, handle all "add" events before "remove" events. Otherwise we might announce new
            // objects and services, which have previously been declared destroyed.

            if let Some(conn) = state.pop_remove_conn() {
                self.shutdown_connection(state, &conn).await;
                continue;
            }

            if let Some((uuid, cookie)) = state.pop_add_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_created_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ObjectCreatedEvent(ObjectCreatedEvent {
                        uuid: uuid.0,
                        cookie: cookie.0,
                        serial: None,
                    })),
                )
                .await;
                continue;
            }

            if let Some((obj_uuid, obj_cookie, uuid, cookie)) = state.pop_add_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_created_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ServiceCreatedEvent(ServiceCreatedEvent {
                        object_uuid: obj_uuid.0,
                        object_cookie: obj_cookie.0,
                        uuid: uuid.0,
                        cookie: cookie.0,
                        serial: None,
                    })),
                )
                .await;
                continue;
            }

            if let Some((obj_uuid, obj_cookie, uuid, cookie)) = state.pop_remove_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_destroyed_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
                        object_uuid: obj_uuid.0,
                        object_cookie: obj_cookie.0,
                        uuid: uuid.0,
                        cookie: cookie.0,
                    })),
                )
                .await;
                continue;
            }

            if let Some((uuid, cookie)) = state.pop_remove_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_destroyed_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
                        uuid: uuid.0,
                        cookie: cookie.0,
                    })),
                )
                .await;
                continue;
            }

            if let Some((serial, conn_id, result)) = state.pop_remove_function_call() {
                let conn = match self.conns.get_mut(&conn_id) {
                    Some(conn) => conn,
                    None => continue,
                };

                let res = conn
                    .send(BrokerEvent::Message(Message::CallFunctionReply(
                        CallFunctionReply { serial, result },
                    )))
                    .await;

                if res.is_err() {
                    state.push_remove_conn(conn_id);
                }
                continue;
            }

            debug_assert!(!state.has_work_left());
            break;
        }
    }

    async fn shutdown_connection(&mut self, state: &mut State, id: &ConnectionId) {
        let mut conn = match self.conns.remove(id) {
            Some(conn) => conn,
            None => return,
        };

        // Ignore errors here
        conn.send(BrokerEvent::Shutdown).await.ok();

        // Remove all objects and services and queue up events.
        for obj_cookie in conn.objects() {
            let obj_uuid = self
                .obj_uuids
                .remove(&obj_cookie)
                .expect("inconsistent state");
            let obj = self.objs.remove(&obj_uuid).expect("inconsistent state");
            state.push_remove_obj(obj_uuid, obj_cookie);

            for svc_cookie in obj.services() {
                let (_, _, svc_uuid) = self
                    .svc_uuids
                    .remove(&svc_cookie)
                    .expect("inconsistent state");
                let svc = self
                    .svcs
                    .remove(&(obj_uuid, svc_uuid))
                    .expect("inconsistent state");
                state.push_remove_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);

                for serial in svc.function_calls() {
                    let (serial, conn_id) = self
                        .function_calls
                        .remove(serial)
                        .expect("inconsistent state");
                    state.push_remove_function_call(
                        serial,
                        conn_id,
                        CallFunctionResult::InvalidService,
                    );
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        msg: Message,
    ) -> Result<(), ()> {
        match msg {
            Message::CreateObject(req) => self.create_object(state, id, req).await,
            Message::DestroyObject(req) => self.destroy_object(state, id, req).await,
            Message::SubscribeObjectsCreated(req) => self.subscribe_objects_created(id, req).await,
            Message::UnsubscribeObjectsCreated => self.unsubscribe_objects_created(id).await,
            Message::SubscribeObjectsDestroyed => self.subscribe_objects_destroyed(id).await,
            Message::UnsubscribeObjectsDestroyed => self.unsubscribe_objects_destroyed(id).await,
            Message::CreateService(req) => self.create_service(state, id, req).await,
            Message::DestroyService(req) => self.destroy_service(state, id, req).await,
            Message::SubscribeServicesCreated(req) => {
                self.subscribe_services_created(id, req).await
            }
            Message::UnsubscribeServicesCreated => self.unsubscribe_services_created(id).await,
            Message::SubscribeServicesDestroyed => self.subscribe_services_destroyed(id).await,
            Message::UnsubscribeServicesDestroyed => self.unsubscribe_services_destroyed(id).await,
            Message::CallFunction(req) => self.call_function(state, id, req).await,
            Message::CallFunctionReply(_) => unimplemented!(),

            Message::Connect(_)
            | Message::ConnectReply(_)
            | Message::CreateObjectReply(_)
            | Message::ObjectCreatedEvent(_)
            | Message::DestroyObjectReply(_)
            | Message::ObjectDestroyedEvent(_)
            | Message::CreateServiceReply(_)
            | Message::ServiceCreatedEvent(_)
            | Message::DestroyServiceReply(_)
            | Message::ServiceDestroyedEvent(_) => Err(()),
        }
    }

    async fn create_object(
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
            Entry::Occupied(_) => {
                conn.send(BrokerEvent::Message(Message::CreateObjectReply(
                    CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::DuplicateId,
                    },
                )))
                .await
            }

            Entry::Vacant(entry) => {
                let cookie = ObjectCookie(Uuid::new_v4());
                conn.send(BrokerEvent::Message(Message::CreateObjectReply(
                    CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::Ok(cookie.0),
                    },
                )))
                .await?;
                let dup = self.obj_uuids.insert(cookie, uuid);
                debug_assert!(dup.is_none());
                entry.insert(Object::new(uuid, cookie, id.clone()));
                conn.add_object(cookie);
                state.push_add_obj(uuid, cookie);
                Ok(())
            }
        }
    }

    async fn destroy_object(
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

        let obj_uuid_entry = match self.obj_uuids.entry(obj_cookie) {
            Entry::Occupied(obj_uuid_entry) => obj_uuid_entry,
            Entry::Vacant(_) => {
                return conn
                    .send(BrokerEvent::Message(Message::DestroyObjectReply(
                        DestroyObjectReply {
                            serial: req.serial,
                            result: DestroyObjectResult::InvalidObject,
                        },
                    )))
                    .await
            }
        };
        let obj_uuid = *obj_uuid_entry.get();

        let entry = match self.objs.entry(obj_uuid) {
            Entry::Occupied(entry) => entry,
            Entry::Vacant(_) => panic!("inconsistent state"),
        };

        if entry.get().conn_id() != id {
            return conn
                .send(BrokerEvent::Message(Message::DestroyObjectReply(
                    DestroyObjectReply {
                        serial: req.serial,
                        result: DestroyObjectResult::ForeignObject,
                    },
                )))
                .await;
        }

        conn.send(BrokerEvent::Message(Message::DestroyObjectReply(
            DestroyObjectReply {
                serial: req.serial,
                result: DestroyObjectResult::Ok,
            },
        )))
        .await?;

        state.push_remove_obj(obj_uuid, obj_cookie);
        for svc_cookie in entry.get().services() {
            let (_, _, svc_uuid) = self
                .svc_uuids
                .remove(&svc_cookie)
                .expect("inconsistent state");
            let svc = self
                .svcs
                .remove(&(obj_uuid, svc_uuid))
                .expect("inconsistent state");
            state.push_remove_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);

            for serial in svc.function_calls() {
                let (serial, conn_id) = self
                    .function_calls
                    .remove(serial)
                    .expect("inconsistent state");
                state.push_remove_function_call(
                    serial,
                    conn_id,
                    CallFunctionResult::InvalidService,
                );
            }
        }

        conn.remove_object(obj_cookie);
        obj_uuid_entry.remove();
        entry.remove();

        Ok(())
    }

    async fn subscribe_objects_created(
        &mut self,
        id: &ConnectionId,
        req: SubscribeObjectsCreated,
    ) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_created_subscribed(true);

        if let Some(serial) = req.serial {
            for (&cookie, &uuid) in &self.obj_uuids {
                conn.send(BrokerEvent::Message(Message::ObjectCreatedEvent(
                    ObjectCreatedEvent {
                        uuid: uuid.0,
                        cookie: cookie.0,
                        serial: Some(serial),
                    },
                )))
                .await?;
            }
        }

        Ok(())
    }

    async fn unsubscribe_objects_created(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_created_subscribed(false);
        Ok(())
    }

    async fn subscribe_objects_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_destroyed_subscribed(true);
        Ok(())
    }

    async fn unsubscribe_objects_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_objects_destroyed_subscribed(false);
        Ok(())
    }

    async fn create_service(
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
                return conn
                    .send(BrokerEvent::Message(Message::CreateServiceReply(
                        CreateServiceReply {
                            serial: req.serial,
                            result: CreateServiceResult::InvalidObject,
                        },
                    )))
                    .await;
            }
        };

        let svc_uuid = ServiceUuid(req.uuid);
        let entry = match self.svcs.entry((obj_uuid, svc_uuid)) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return conn
                    .send(BrokerEvent::Message(Message::CreateServiceReply(
                        CreateServiceReply {
                            serial: req.serial,
                            result: CreateServiceResult::DuplicateId,
                        },
                    )))
                    .await;
            }
        };

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return conn
                .send(BrokerEvent::Message(Message::CreateServiceReply(
                    CreateServiceReply {
                        serial: req.serial,
                        result: CreateServiceResult::ForeignObject,
                    },
                )))
                .await;
        }

        let svc_cookie = ServiceCookie(Uuid::new_v4());
        conn.send(BrokerEvent::Message(Message::CreateServiceReply(
            CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::Ok(svc_cookie.0),
            },
        )))
        .await?;

        let dup = self
            .svc_uuids
            .insert(svc_cookie, (obj_uuid, obj_cookie, svc_uuid));
        debug_assert!(dup.is_none());
        entry.insert(Service::new(obj_uuid, obj_cookie, svc_uuid, svc_cookie));
        obj.add_service(svc_cookie);
        state.push_add_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);

        Ok(())
    }

    async fn destroy_service(
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
        let svc_uuid_entry = match self.svc_uuids.entry(svc_cookie) {
            Entry::Occupied(svc_uuid_entry) => svc_uuid_entry,
            Entry::Vacant(_) => {
                return conn
                    .send(BrokerEvent::Message(Message::DestroyServiceReply(
                        DestroyServiceReply {
                            serial: req.serial,
                            result: DestroyServiceResult::InvalidService,
                        },
                    )))
                    .await;
            }
        };
        let (obj_uuid, obj_cookie, svc_uuid) = *svc_uuid_entry.get();

        let obj = self.objs.get_mut(&obj_uuid).expect("inconsistent state");
        if obj.conn_id() != id {
            return conn
                .send(BrokerEvent::Message(Message::DestroyServiceReply(
                    DestroyServiceReply {
                        serial: req.serial,
                        result: DestroyServiceResult::ForeignObject,
                    },
                )))
                .await;
        }

        conn.send(BrokerEvent::Message(Message::DestroyServiceReply(
            DestroyServiceReply {
                serial: req.serial,
                result: DestroyServiceResult::Ok,
            },
        )))
        .await?;

        let entry = match self.svcs.entry((obj_uuid, svc_uuid)) {
            Entry::Occupied(entry) => entry,
            Entry::Vacant(_) => panic!("inconsistent state"),
        };

        state.push_remove_svc(obj_uuid, obj_cookie, svc_uuid, svc_cookie);
        obj.remove_service(svc_cookie);

        for serial in entry.get().function_calls() {
            let (serial, conn_id) = self
                .function_calls
                .remove(serial)
                .expect("inconsistent state");
            state.push_remove_function_call(serial, conn_id, CallFunctionResult::InvalidService);
        }

        svc_uuid_entry.remove();
        entry.remove();
        Ok(())
    }

    async fn subscribe_services_created(
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
                conn.send(BrokerEvent::Message(Message::ServiceCreatedEvent(
                    ServiceCreatedEvent {
                        object_uuid: obj_uuid.0,
                        object_cookie: obj_cookie.0,
                        uuid: svc_uuid.0,
                        cookie: svc_cookie.0,
                        serial: Some(serial),
                    },
                )))
                .await?;
            }
        }

        Ok(())
    }

    async fn unsubscribe_services_created(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_created_subscribed(false);
        Ok(())
    }

    async fn subscribe_services_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_destroyed_subscribed(true);
        Ok(())
    }

    async fn unsubscribe_services_destroyed(&mut self, id: &ConnectionId) -> Result<(), ()> {
        let conn = match self.conns.get_mut(id) {
            Some(conn) => conn,
            None => return Ok(()),
        };

        conn.set_services_destroyed_subscribed(false);
        Ok(())
    }

    async fn call_function(
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

                return conn
                    .send(BrokerEvent::Message(Message::CallFunctionReply(
                        CallFunctionReply {
                            serial: req.serial,
                            result: CallFunctionResult::InvalidService,
                        },
                    )))
                    .await;
            }
        };

        let callee_id = match self.objs.get(&obj_uuid) {
            Some(obj) => obj.conn_id(),
            None => panic!("inconsistent state"),
        };
        let callee_conn = self.conns.get_mut(callee_id).expect("inconsistent state");

        let serial = self.function_calls.insert((req.serial, id.clone()));
        let res = callee_conn
            .send(BrokerEvent::Message(Message::CallFunction(CallFunction {
                serial,
                service_cookie: svc_cookie.0,
                function: req.function,
                args: req.args,
            })))
            .await;

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
}

async fn broadcast<'a, I>(conns: I, state: &mut State, ev: BrokerEvent)
where
    // This should really be IntoIterator, but we're hitting a compiler bug:
    // https://github.com/tokio-rs/tokio/issues/1835
    // https://github.com/rust-lang/rust/issues/60658
    I: Iterator<Item = (&'a ConnectionId, &'a mut ConnectionState)>,
{
    for (id, conn) in conns {
        // Shutdown connection on error, but don't abort loop.
        if let Err(()) = conn.send(ev.clone()).await {
            state.push_remove_conn(id.clone());
        }
    }
}
