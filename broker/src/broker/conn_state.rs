use aldrin_proto::message::Message;
use aldrin_proto::{BusListenerCookie, ChannelCookie, ObjectCookie, ServiceCookie};
use futures_channel::mpsc::UnboundedSender;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct ConnectionState {
    send: UnboundedSender<Message>,
    objects: HashSet<ObjectCookie>,

    /// Map of active subscriptions made by this connection.
    subscriptions: HashMap<ServiceCookie, HashSet<u32>>,

    senders: HashSet<ChannelCookie>,
    receivers: HashSet<ChannelCookie>,
    bus_listeners: HashSet<BusListenerCookie>,
}

impl ConnectionState {
    pub fn new(send: UnboundedSender<Message>) -> Self {
        ConnectionState {
            send,
            objects: HashSet::new(),
            subscriptions: HashMap::new(),
            senders: HashSet::new(),
            receivers: HashSet::new(),
            bus_listeners: HashSet::new(),
        }
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
}
