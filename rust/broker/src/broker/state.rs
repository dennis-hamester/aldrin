use crate::conn_id::ConnectionId;
use aldrin_proto::{CallFunctionResult, ObjectCookie, ObjectUuid, ServiceCookie, ServiceUuid};

#[derive(Debug)]
pub(super) struct State {
    shutdown_now: bool,
    shutdown_idle: bool,
    add_objs: Vec<(ObjectUuid, ObjectCookie)>,
    remove_conns: Vec<ConnectionId>,
    remove_objs: Vec<(ObjectUuid, ObjectCookie)>,
    add_svcs: Vec<(ObjectUuid, ObjectCookie, ServiceUuid, ServiceCookie)>,
    remove_svcs: Vec<(ObjectUuid, ObjectCookie, ServiceUuid, ServiceCookie)>,
    remove_function_calls: Vec<(u32, ConnectionId, CallFunctionResult)>,
    remove_subscriptions: Vec<(
        ConnectionId,
        ObjectUuid,
        ObjectCookie,
        ServiceUuid,
        ServiceCookie,
    )>,
    unsubscribe: Vec<(ConnectionId, ServiceCookie, u32)>,
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
            remove_function_calls: Vec::new(),
            remove_subscriptions: Vec::new(),
            unsubscribe: Vec::new(),
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
            || !self.remove_function_calls.is_empty()
            || !self.remove_subscriptions.is_empty()
            || !self.unsubscribe.is_empty()
    }

    pub fn push_add_obj(&mut self, uuid: ObjectUuid, cookie: ObjectCookie) {
        self.add_objs.push((uuid, cookie));
    }

    pub fn pop_add_obj(&mut self) -> Option<(ObjectUuid, ObjectCookie)> {
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

    pub fn push_remove_obj(&mut self, uuid: ObjectUuid, cookie: ObjectCookie) {
        self.remove_objs.push((uuid, cookie));
    }

    pub fn pop_remove_obj(&mut self) -> Option<(ObjectUuid, ObjectCookie)> {
        self.remove_objs.pop()
    }

    pub fn push_add_svc(
        &mut self,
        object_uuid: ObjectUuid,
        object_cookie: ObjectCookie,
        uuid: ServiceUuid,
        cookie: ServiceCookie,
    ) {
        self.add_svcs
            .push((object_uuid, object_cookie, uuid, cookie));
    }

    pub fn pop_add_svc(
        &mut self,
    ) -> Option<(ObjectUuid, ObjectCookie, ServiceUuid, ServiceCookie)> {
        self.add_svcs.pop()
    }

    pub fn push_remove_svc(
        &mut self,
        object_uuid: ObjectUuid,
        object_cookie: ObjectCookie,
        uuid: ServiceUuid,
        cookie: ServiceCookie,
    ) {
        self.remove_svcs
            .push((object_uuid, object_cookie, uuid, cookie));
    }

    pub fn pop_remove_svc(
        &mut self,
    ) -> Option<(ObjectUuid, ObjectCookie, ServiceUuid, ServiceCookie)> {
        self.remove_svcs.pop()
    }

    pub fn push_remove_function_call(
        &mut self,
        serial: u32,
        conn_id: ConnectionId,
        res: CallFunctionResult,
    ) {
        self.remove_function_calls.push((serial, conn_id, res));
    }

    pub fn pop_remove_function_call(&mut self) -> Option<(u32, ConnectionId, CallFunctionResult)> {
        self.remove_function_calls.pop()
    }

    pub fn push_remove_subscriptions(
        &mut self,
        conn_id: ConnectionId,
        obj_uuid: ObjectUuid,
        obj_cookie: ObjectCookie,
        svc_uuid: ServiceUuid,
        svc_cookie: ServiceCookie,
    ) {
        self.remove_subscriptions
            .push((conn_id, obj_uuid, obj_cookie, svc_uuid, svc_cookie));
    }

    pub fn pop_remove_subscriptions(
        &mut self,
    ) -> Option<(
        ConnectionId,
        ObjectUuid,
        ObjectCookie,
        ServiceUuid,
        ServiceCookie,
    )> {
        self.remove_subscriptions.pop()
    }

    pub fn push_unsubscribe(
        &mut self,
        conn_id: ConnectionId,
        svc_cookie: ServiceCookie,
        event: u32,
    ) {
        self.unsubscribe.push((conn_id, svc_cookie, event));
    }

    pub fn pop_unsubscribe(&mut self) -> Option<(ConnectionId, ServiceCookie, u32)> {
        self.unsubscribe.pop()
    }
}
