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

use crate::conn_id::ConnectionId;
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct State {
    shutdown_now: bool,
    shutdown_idle: bool,
    add_objs: Vec<Uuid>,
    remove_conns: Vec<ConnectionId>,
    remove_objs: Vec<Uuid>,
    add_svcs: Vec<(Uuid, Uuid)>,
    remove_svcs: Vec<(Uuid, Uuid)>,
}

impl State {
    pub fn new() -> Self {
        State {
            shutdown_now: false,
            shutdown_idle: false,
            add_objs: Vec::new(),
            remove_conns: Vec::new(),
            remove_objs: Vec::new(),
            add_svcs: Vec::new(),
            remove_svcs: Vec::new(),
        }
    }

    pub fn set_shutdown_now(&mut self) {
        self.shutdown_now = true;
    }

    pub fn shutdown_now(&self) -> bool {
        self.shutdown_now
    }

    pub fn set_shutdown_idle(&mut self) {
        self.shutdown_idle = true;
    }

    pub fn shutdown_idle(&self) -> bool {
        self.shutdown_idle
    }

    pub fn has_work_left(&self) -> bool {
        !self.add_objs.is_empty()
            || !self.remove_conns.is_empty()
            || !self.remove_objs.is_empty()
            || !self.add_svcs.is_empty()
            || !self.remove_svcs.is_empty()
    }

    pub fn push_add_obj(&mut self, id: Uuid) {
        self.add_objs.push(id);
    }

    pub fn pop_add_obj(&mut self) -> Option<Uuid> {
        self.add_objs.pop()
    }

    pub fn push_remove_conn(&mut self, id: ConnectionId) {
        self.remove_conns.push(id);
    }

    pub fn push_remove_conns<I>(&mut self, ids: I)
    where
        I: IntoIterator<Item = ConnectionId>,
    {
        self.remove_conns.extend(ids);
    }

    pub fn pop_remove_conn(&mut self) -> Option<ConnectionId> {
        self.remove_conns.pop()
    }

    pub fn push_remove_obj(&mut self, id: Uuid) {
        self.remove_objs.push(id);
    }

    pub fn pop_remove_obj(&mut self) -> Option<Uuid> {
        self.remove_objs.pop()
    }

    pub fn push_add_svc(&mut self, object_id: Uuid, id: Uuid) {
        self.add_svcs.push((object_id, id));
    }

    pub fn pop_add_svc(&mut self) -> Option<(Uuid, Uuid)> {
        self.add_svcs.pop()
    }

    pub fn push_remove_svc(&mut self, object_id: Uuid, id: Uuid) {
        self.remove_svcs.push((object_id, id));
    }

    pub fn pop_remove_svc(&mut self) -> Option<(Uuid, Uuid)> {
        self.remove_svcs.pop()
    }
}
