use crate::conn_id::ConnectionId;
use crate::core::message::Message;
use crate::core::{BusListenerCookie, ChannelCookie, ObjectCookie, ProtocolVersion, ServiceCookie};
use futures_channel::mpsc::UnboundedSender;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct ConnectionState {
    protocol_version: ProtocolVersion,
    send: UnboundedSender<Message>,
    objects: HashSet<ObjectCookie>,

    /// Map of active subscriptions made by this connection.
    subscriptions: HashMap<ServiceCookie, HashSet<u32>>,

    senders: HashSet<ChannelCookie>,
    receivers: HashSet<ChannelCookie>,
    bus_listeners: HashSet<BusListenerCookie>,
    calls: HashMap<u32, (u32, ConnectionId)>,
}

impl ConnectionState {
    pub fn new(protocol_version: ProtocolVersion, send: UnboundedSender<Message>) -> Self {
        ConnectionState {
            protocol_version,
            send,
            objects: HashSet::new(),
            subscriptions: HashMap::new(),
            senders: HashSet::new(),
            receivers: HashSet::new(),
            bus_listeners: HashSet::new(),
            calls: HashMap::new(),
        }
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn add_object(&mut self, cookie: ObjectCookie) {
        let unique = self.objects.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_object(&mut self, cookie: ObjectCookie) {
        let contained = self.objects.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn objects(&self) -> impl Iterator<Item = ObjectCookie> + '_ {
        self.objects.iter().copied()
    }

    pub fn send(&self, msg: Message) -> Result<(), ()> {
        self.send.unbounded_send(msg).map_err(|_| ())
    }

    pub fn add_subscription(&mut self, svc_cookie: ServiceCookie, id: u32) {
        self.subscriptions.entry(svc_cookie).or_default().insert(id);
    }

    pub fn remove_subscription(&mut self, svc_cookie: ServiceCookie, id: u32) {
        if let Entry::Occupied(mut subs) = self.subscriptions.entry(svc_cookie) {
            subs.get_mut().remove(&id);
            if subs.get().is_empty() {
                subs.remove();
            }
        }
    }

    pub fn remove_all_subscriptions(&mut self, svc_cookie: ServiceCookie) {
        self.subscriptions.remove(&svc_cookie);
    }

    pub fn subscriptions(&self) -> impl Iterator<Item = (ServiceCookie, u32)> + '_ {
        self.subscriptions
            .iter()
            .flat_map(|(&c, ids)| ids.iter().map(move |&id| (c, id)))
    }

    pub fn is_subscribed_to(&self, svc_cookie: ServiceCookie, id: u32) -> bool {
        self.subscriptions
            .get(&svc_cookie)
            .map(|s| s.contains(&id))
            .unwrap_or(false)
    }

    pub fn add_sender(&mut self, cookie: ChannelCookie) {
        let unique = self.senders.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_sender(&mut self, cookie: ChannelCookie) {
        let contained = self.senders.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn senders(&self) -> impl Iterator<Item = ChannelCookie> + '_ {
        self.senders.iter().copied()
    }

    pub fn add_receiver(&mut self, cookie: ChannelCookie) {
        let unique = self.receivers.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_receiver(&mut self, cookie: ChannelCookie) {
        let contained = self.receivers.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn receivers(&self) -> impl Iterator<Item = ChannelCookie> + '_ {
        self.receivers.iter().copied()
    }

    pub fn add_bus_listener(&mut self, cookie: BusListenerCookie) {
        let unique = self.bus_listeners.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_bus_listener(&mut self, cookie: BusListenerCookie) {
        let contained = self.bus_listeners.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn bus_listeners(&self) -> impl Iterator<Item = BusListenerCookie> + '_ {
        self.bus_listeners.iter().copied()
    }

    pub fn add_call(
        &mut self,
        caller_serial: u32,
        callee_serial: u32,
        callee_id: ConnectionId,
    ) -> bool {
        match self.calls.entry(caller_serial) {
            Entry::Occupied(_) => false,

            Entry::Vacant(entry) => {
                entry.insert((callee_serial, callee_id));
                true
            }
        }
    }

    pub fn remove_call(&mut self, caller_serial: u32) {
        let call = self.calls.remove(&caller_serial);
        debug_assert!(call.is_some());
    }

    pub fn call_data(&self, caller_serial: u32) -> Option<(u32, &ConnectionId)> {
        self.calls
            .get(&caller_serial)
            .map(|(callee_serial, callee_id)| (*callee_serial, callee_id))
    }

    pub fn calls(&self) -> impl Iterator<Item = (u32, &ConnectionId)> {
        self.calls
            .values()
            .map(|(callee_serial, callee_id)| (*callee_serial, callee_id))
    }
}
