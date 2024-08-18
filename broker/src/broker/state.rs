use crate::conn_id::ConnectionId;
use crate::core::message::CallFunctionResult;
use crate::core::{ObjectId, ServiceCookie, ServiceId};

#[derive(Debug)]
pub(super) struct State {
    shutdown_now: bool,
    shutdown_idle: bool,
    remove_conns: Vec<(ConnectionId, bool)>,
    remove_function_calls: Vec<(u32, ConnectionId, CallFunctionResult)>,
    services_destroyed: Vec<(ConnectionId, ServiceCookie)>,
    unsubscribe_event: Vec<(ConnectionId, ServiceCookie, u32)>,
    unsubscribe_all_events: Vec<(ConnectionId, ServiceCookie)>,
    create_object: Vec<ObjectId>,
    destroy_object: Vec<ObjectId>,
    create_service: Vec<ServiceId>,
    destroy_service: Vec<ServiceId>,
    abort_function_calls: Vec<(u32, ConnectionId)>,
}

impl State {
    pub fn new() -> Self {
        State {
            shutdown_now: false,
            shutdown_idle: false,
            remove_conns: Vec::new(),
            remove_function_calls: Vec::new(),
            services_destroyed: Vec::new(),
            unsubscribe_event: Vec::new(),
            unsubscribe_all_events: Vec::new(),
            create_object: Vec::new(),
            destroy_object: Vec::new(),
            create_service: Vec::new(),
            destroy_service: Vec::new(),
            abort_function_calls: Vec::new(),
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
        !self.remove_conns.is_empty()
            || !self.remove_function_calls.is_empty()
            || !self.services_destroyed.is_empty()
            || !self.unsubscribe_event.is_empty()
            || !self.unsubscribe_all_events.is_empty()
            || !self.create_object.is_empty()
            || !self.destroy_object.is_empty()
            || !self.create_service.is_empty()
            || !self.destroy_service.is_empty()
            || !self.abort_function_calls.is_empty()
    }

    pub fn push_remove_conn(&mut self, id: ConnectionId, send_shutdown: bool) {
        self.remove_conns.push((id, send_shutdown));
    }

    pub fn push_remove_conns<I>(&mut self, ids: I)
    where
        I: IntoIterator<Item = (ConnectionId, bool)>,
    {
        self.remove_conns.extend(ids);
    }

    pub fn pop_remove_conn(&mut self) -> Option<(ConnectionId, bool)> {
        self.remove_conns.pop()
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

    pub fn push_services_destroyed(&mut self, conn_id: ConnectionId, svc_cookie: ServiceCookie) {
        self.services_destroyed.push((conn_id, svc_cookie));
    }

    pub fn pop_services_destroyed(&mut self) -> Option<(ConnectionId, ServiceCookie)> {
        self.services_destroyed.pop()
    }

    pub fn push_unsubscribe_event(
        &mut self,
        conn_id: ConnectionId,
        svc_cookie: ServiceCookie,
        event: u32,
    ) {
        self.unsubscribe_event.push((conn_id, svc_cookie, event));
    }

    pub fn pop_unsubscribe_event(&mut self) -> Option<(ConnectionId, ServiceCookie, u32)> {
        self.unsubscribe_event.pop()
    }

    pub fn push_unsubscribe_all_events(
        &mut self,
        conn_id: ConnectionId,
        svc_cookie: ServiceCookie,
    ) {
        self.unsubscribe_all_events.push((conn_id, svc_cookie));
    }

    pub fn pop_unsubscribe_all_events(&mut self) -> Option<(ConnectionId, ServiceCookie)> {
        self.unsubscribe_all_events.pop()
    }

    pub fn push_create_object(&mut self, object: ObjectId) {
        self.create_object.push(object);
    }

    pub fn pop_create_object(&mut self) -> Option<ObjectId> {
        self.create_object.pop()
    }

    pub fn push_destroy_object(&mut self, object: ObjectId) {
        self.destroy_object.push(object);
    }

    pub fn pop_destroy_object(&mut self) -> Option<ObjectId> {
        self.destroy_object.pop()
    }

    pub fn push_create_service(&mut self, service: ServiceId) {
        self.create_service.push(service);
    }

    pub fn pop_create_service(&mut self) -> Option<ServiceId> {
        self.create_service.pop()
    }

    pub fn push_destroy_service(&mut self, service: ServiceId) {
        self.destroy_service.push(service);
    }

    pub fn pop_destroy_service(&mut self) -> Option<ServiceId> {
        self.destroy_service.pop()
    }

    pub fn push_abort_function_call(&mut self, callee_serial: u32, callee_id: ConnectionId) {
        self.abort_function_calls.push((callee_serial, callee_id));
    }

    pub fn pop_abort_function_call(&mut self) -> Option<(u32, ConnectionId)> {
        self.abort_function_calls.pop()
    }
}
