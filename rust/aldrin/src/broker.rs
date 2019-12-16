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
mod state;

use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionId;
use crate::proto::broker::*;
use crate::proto::client::*;
use conn_state::ConnectionState;
use futures_channel::mpsc::{channel, Receiver};
use futures_util::stream::StreamExt;
use object::Object;
use state::State;
use std::collections::hash_map::{Entry, HashMap};
use uuid::Uuid;

pub use builder::Builder;
pub use error::Error;
pub use handle::Handle;

pub(crate) use event::BrokerEvent;

#[derive(Debug)]
pub struct Broker {
    recv: Receiver<ConnectionEvent>,
    handle: Option<Handle>,
    conns: HashMap<ConnectionId, ConnectionState>,
    objs: HashMap<Uuid, Object>,
}

impl Broker {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub(crate) fn new(fifo_size: usize) -> Self {
        let (send, recv) = channel(fifo_size);

        Broker {
            recv,
            handle: Some(Handle::new(send)),
            conns: HashMap::new(),
            objs: HashMap::new(),
        }
    }

    pub fn handle(&self) -> &Handle {
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

            ConnectionEvent::ClientMessage(id, msg) => {
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
            if let Some(conn) = state.pop_remove_conn() {
                self.shutdown_connection(state, &conn).await;
                continue;
            }

            if let Some(_) = state.pop_remove_obj() {
                continue;
            }

            if let Some(id) = state.pop_add_obj() {
                broadcast(
                    self.conns
                        .iter_mut()
                        .filter(|(_, c)| c.objects_created_subscribed()),
                    state,
                    BrokerEvent::BrokerMessage(BrokerMessage::ObjectCreatedEvent(
                        ObjectCreatedEvent { id, serial: None },
                    )),
                )
                .await;
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

        // Mark all owned objects as destroyed
        state.push_remove_objs(conn.objects());
    }

    async fn handle_message(
        &mut self,
        state: &mut State,
        id: &ConnectionId,
        msg: ClientMessage,
    ) -> Result<(), ()> {
        match msg {
            ClientMessage::CreateObject(req) => self.create_object(state, id, req).await,
            ClientMessage::DestroyObject(req) => self.destroy_object(state, id, req).await,
            ClientMessage::SubscribeObjectsCreated(req) => {
                self.subscribe_objects_created(id, req).await
            }
            ClientMessage::UnsubscribeObjectsCreated => self.unsubscribe_objects_created(id).await,
            ClientMessage::SubscribeObjectsDestroyed => unimplemented!(),
            ClientMessage::Connect(_) => Err(()),
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
                conn.send(BrokerEvent::BrokerMessage(
                    BrokerMessage::CreateObjectReply(CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::DuplicateId,
                    }),
                ))
                .await
            }

            Entry::Vacant(entry) => {
                conn.send(BrokerEvent::BrokerMessage(
                    BrokerMessage::CreateObjectReply(CreateObjectReply {
                        serial: req.serial,
                        result: CreateObjectResult::Ok,
                    }),
                ))
                .await?;
                entry.insert(Object::new(req.id, id.clone()));
                conn.add_object(req.id);
                state.push_add_obj(req.id);
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

        if let Entry::Occupied(entry) = self.objs.entry(req.id) {
            if entry.get().conn_id() == id {
                conn.send(BrokerEvent::BrokerMessage(
                    BrokerMessage::DestroyObjectReply(DestroyObjectReply {
                        serial: req.serial,
                        result: DestroyObjectResult::Ok,
                    }),
                ))
                .await?;
                entry.remove();
                conn.remove_object(req.id);
                state.push_remove_obj(req.id);
                Ok(())
            } else {
                conn.send(BrokerEvent::BrokerMessage(
                    BrokerMessage::DestroyObjectReply(DestroyObjectReply {
                        serial: req.serial,
                        result: DestroyObjectResult::ForeignObject,
                    }),
                ))
                .await
            }
        } else {
            conn.send(BrokerEvent::BrokerMessage(
                BrokerMessage::DestroyObjectReply(DestroyObjectReply {
                    serial: req.serial,
                    result: DestroyObjectResult::InvalidObject,
                }),
            ))
            .await
        }
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
            for &id in self.objs.keys() {
                conn.send(BrokerEvent::BrokerMessage(
                    BrokerMessage::ObjectCreatedEvent(ObjectCreatedEvent {
                        id,
                        serial: Some(serial),
                    }),
                ))
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
