use aldrin_proto::{Message, ObjectCookie, ServiceCookie};
use futures_channel::mpsc::UnboundedSender;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct ConnectionState {
    send: UnboundedSender<Message>,
    objects: HashSet<ObjectCookie>,
    objects_subscribed: bool,
    services_subscribed: bool,

    /// Map of active subscriptions made by this connection.
    subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
}

impl ConnectionState {
    pub fn new(send: UnboundedSender<Message>) -> Self {
        ConnectionState {
            send,
            objects: HashSet::new(),
            objects_subscribed: false,
            services_subscribed: false,
            subscriptions: HashMap::new(),
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

    pub fn set_objects_subscribed(&mut self, subscribed: bool) {
        self.objects_subscribed = subscribed;
    }

    pub fn objects_subscribed(&self) -> bool {
        self.objects_subscribed
    }

    pub fn set_services_subscribed(&mut self, subscribed: bool) {
        self.services_subscribed = subscribed;
    }

    pub fn services_subscribed(&self) -> bool {
        self.services_subscribed
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
}
