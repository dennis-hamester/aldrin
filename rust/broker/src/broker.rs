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
use object::Object;
use service::Service;
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
    objs: HashMap<Uuid, Object>,
    svcs: HashMap<(Uuid, Uuid), Service>,
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
            objs: HashMap::new(),
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

            if let Some((id, cookie)) = state.pop_add_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_created_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ObjectCreatedEvent(ObjectCreatedEvent {
                        id,
                        cookie,
                        serial: None,
                    })),
                )
                .await;
                continue;
            }

            if let Some((object_id, object_cookie, id, cookie)) = state.pop_add_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_created_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ServiceCreatedEvent(ServiceCreatedEvent {
                        object_id,
                        object_cookie,
                        id,
                        cookie,
                        serial: None,
                    })),
                )
                .await;
                continue;
            }

            if let Some((object_id, object_cookie, id, cookie)) = state.pop_remove_svc() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.services_destroyed_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
                        object_id,
                        object_cookie,
                        id,
                        cookie,
                    })),
                )
                .await;
                continue;
            }

            if let Some((id, cookie)) = state.pop_remove_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_destroyed_subscribed()),
                    state,
                    BrokerEvent::Message(Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
                        id,
                        cookie,
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
        for obj_id in conn.objects() {
            let obj = self.objs.remove(&obj_id).expect("inconsistent state");
            state.push_remove_obj(obj_id, obj.cookie());

            for svc_id in obj.services() {
                let svc = self
                    .svcs
                    .remove(&(obj_id, svc_id))
                    .expect("inconsistent state");
                state.push_remove_svc(obj_id, obj.cookie(), svc_id, svc.cookie());

                for serial in svc.function_calls() {
                    let (serial, conn_id) = self
                        .function_calls
                        .remove(serial)
                        .expect("inconsistent state");
                    state.push_remove_function_call(
                        serial,
                        conn_id,
                        CallFunctionResult::InvalidObject,
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

        match self.objs.entry(req.id) {
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
                let cookie = Uuid::new_v4();
                conn.send(BrokerEvent::Message(Message::CreateObjectReply(
                    CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::Ok(cookie),
                    },
                )))
                .await?;
                entry.insert(Object::new(req.id, cookie, id.clone()));
                conn.add_object(req.id);
                state.push_add_obj(req.id, cookie);
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

        let entry = match self.objs.entry(req.id) {
            Entry::Occupied(entry) if entry.get().cookie() == req.cookie => entry,
            _ => {
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

        let obj_cookie = entry.get().cookie();
        state.push_remove_obj(req.id, obj_cookie);
        for svc_id in entry.get().services() {
            let svc = self
                .svcs
                .remove(&(req.id, svc_id))
                .expect("inconsistent state");
            state.push_remove_svc(req.id, obj_cookie, svc_id, svc.cookie());

            for serial in svc.function_calls() {
                let (serial, conn_id) = self
                    .function_calls
                    .remove(serial)
                    .expect("inconsistent state");
                state.push_remove_function_call(serial, conn_id, CallFunctionResult::InvalidObject);
            }
        }

        conn.remove_object(req.id);
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
            for (&id, obj) in &self.objs {
                conn.send(BrokerEvent::Message(Message::ObjectCreatedEvent(
                    ObjectCreatedEvent {
                        id,
                        cookie: obj.cookie(),
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

        let entry = match self.svcs.entry((req.object_id, req.id)) {
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

        let obj = match self.objs.get_mut(&req.object_id) {
            Some(obj) if obj.cookie() == req.object_cookie => obj,
            _ => {
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

        let cookie = Uuid::new_v4();
        conn.send(BrokerEvent::Message(Message::CreateServiceReply(
            CreateServiceReply {
                serial: req.serial,
                result: CreateServiceResult::Ok(cookie),
            },
        )))
        .await?;

        entry.insert(Service::new(
            req.object_id,
            req.object_cookie,
            req.id,
            cookie,
        ));
        obj.add_service(req.id);
        state.push_add_svc(req.object_id, obj.cookie(), req.id, cookie);

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

        let obj = match self.objs.get_mut(&req.object_id) {
            Some(obj) => obj,
            None => {
                return conn
                    .send(BrokerEvent::Message(Message::DestroyServiceReply(
                        DestroyServiceReply {
                            serial: req.serial,
                            result: DestroyServiceResult::InvalidObject,
                        },
                    )))
                    .await;
            }
        };

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

        let entry = match self.svcs.entry((req.object_id, req.id)) {
            Entry::Occupied(entry) if entry.get().cookie() == req.cookie => entry,
            _ => {
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

        conn.send(BrokerEvent::Message(Message::DestroyServiceReply(
            DestroyServiceReply {
                serial: req.serial,
                result: DestroyServiceResult::Ok,
            },
        )))
        .await?;

        state.push_remove_svc(req.object_id, obj.cookie(), req.id, req.cookie);
        obj.remove_service(req.id);

        for serial in entry.get().function_calls() {
            let (serial, conn_id) = self
                .function_calls
                .remove(serial)
                .expect("inconsistent state");
            state.push_remove_function_call(serial, conn_id, CallFunctionResult::InvalidService);
        }

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
            for (&(object_id, id), svc) in &self.svcs {
                conn.send(BrokerEvent::Message(Message::ServiceCreatedEvent(
                    ServiceCreatedEvent {
                        object_id,
                        object_cookie: svc.object_cookie(),
                        id,
                        cookie: svc.cookie(),
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
        let obj = match self.objs.get(&req.object_id) {
            Some(obj) => obj,
            None => {
                let conn = match self.conns.get_mut(id) {
                    Some(conn) => conn,
                    None => return Ok(()),
                };

                return conn
                    .send(BrokerEvent::Message(Message::CallFunctionReply(
                        CallFunctionReply {
                            serial: req.serial,
                            result: CallFunctionResult::InvalidObject,
                        },
                    )))
                    .await;
            }
        };

        let svc = match self.svcs.get_mut(&(req.object_id, req.service_id)) {
            Some(svc) if svc.cookie() == req.service_cookie => svc,
            _ => {
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

        let serial = self.function_calls.insert((req.serial, id.clone()));
        let callee_id = obj.conn_id();
        let callee_conn = self
            .conns
            .get_mut(callee_id)
            .expect("invalid connection id");

        let res = callee_conn
            .send(BrokerEvent::Message(Message::CallFunction(CallFunction {
                serial,
                object_id: req.object_id,
                service_id: req.service_id,
                service_cookie: req.service_cookie,
                function: req.function,
                args: req.args,
            })))
            .await;

        if res.is_ok() {
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
