use crate::conn_id::ConnectionId;
use aldrin_core::message::Message;
use aldrin_core::{BusListenerCookie, ChannelCookie, ObjectCookie, ProtocolVersion, ServiceCookie};
use futures_channel::mpsc::UnboundedSender;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct ConnectionState {
    protocol_version: ProtocolVersion,
    send: UnboundedSender<Message>,
    objects: HashSet<ObjectCookie>,
    events: HashMap<ServiceCookie, HashSet<u32>>,
    all_events: HashSet<ServiceCookie>,
    subscriptions: HashSet<ServiceCookie>,
    senders: HashSet<ChannelCookie>,
    receivers: HashSet<ChannelCookie>,
    bus_listeners: HashSet<BusListenerCookie>,
    calls: HashMap<u32, (u32, ConnectionId)>,
}

impl ConnectionState {
    pub fn new(protocol_version: ProtocolVersion, send: UnboundedSender<Message>) -> Self {
        Self {
            protocol_version,
            send,
            objects: HashSet::new(),
            events: HashMap::new(),
            all_events: HashSet::new(),
            subscriptions: HashSet::new(),
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

    pub fn subscribe_event(&mut self, svc_cookie: ServiceCookie, event: u32) {
        self.events.entry(svc_cookie).or_default().insert(event);
    }

    pub fn unsubscribe_event(&mut self, svc_cookie: ServiceCookie, event: u32) {
        if let Entry::Occupied(mut subs) = self.events.entry(svc_cookie) {
            subs.get_mut().remove(&event);
            if subs.get().is_empty() {
                subs.remove();
            }
        }
    }

    pub fn event_subscriptions(&self) -> impl Iterator<Item = (ServiceCookie, u32)> + '_ {
        self.events
            .iter()
            .flat_map(|(&c, ids)| ids.iter().map(move |&event| (c, event)))
    }

    pub fn is_subscribed_to_event(&self, svc_cookie: ServiceCookie, event: u32) -> bool {
        self.all_events.contains(&svc_cookie)
            || self
                .events
                .get(&svc_cookie)
                .map(|s| s.contains(&event))
                .unwrap_or(false)
    }

    pub fn subscribe_all_events(&mut self, svc_cookie: ServiceCookie) {
        self.all_events.insert(svc_cookie);
    }

    pub fn unsubscribe_all_events(&mut self, svc_cookie: ServiceCookie) {
        self.all_events.remove(&svc_cookie);
    }

    pub fn all_event_subscriptions(&self) -> impl Iterator<Item = ServiceCookie> + '_ {
        self.all_events.iter().copied()
    }

    pub fn subscribe(&mut self, svc_cookie: ServiceCookie) {
        self.subscriptions.insert(svc_cookie);
    }

    pub fn unsubscribe(&mut self, svc_cookie: ServiceCookie) {
        self.subscriptions.remove(&svc_cookie);
    }

    pub fn subscriptions(&self) -> impl Iterator<Item = ServiceCookie> + '_ {
        self.subscriptions.iter().copied()
    }

    pub fn unsubscribe_all(&mut self, svc_cookie: ServiceCookie) {
        self.events.remove(&svc_cookie);
        self.subscriptions.remove(&svc_cookie);
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
