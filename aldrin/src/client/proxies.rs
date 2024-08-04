use crate::handle::Handle;
use crate::low_level::{Event, Proxy, ProxyId};
use aldrin_core::{SerializedValue, ServiceCookie, ServiceId, ServiceInfo};
use futures_channel::mpsc::{self, UnboundedSender};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Proxies {
    entries: HashMap<ProxyId, ProxyEntry>,
    services: HashMap<ServiceCookie, HashSet<ProxyId>>,
}

impl Proxies {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            services: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        client: Handle,
        service: ServiceId,
        info: ServiceInfo,
    ) -> (Proxy, bool) {
        let id = ProxyId::new_v4();
        let (send, recv) = mpsc::unbounded();

        self.entries
            .insert(id, ProxyEntry::new(service.cookie, send));

        let subscribe_service = match self.services.entry(service.cookie) {
            Entry::Occupied(mut entries) => {
                debug_assert!(!entries.get().is_empty());
                entries.get_mut().insert(id);
                false
            }

            Entry::Vacant(entries) => {
                let entries = entries.insert(HashSet::new());
                entries.insert(id);
                true
            }
        };

        (
            Proxy::new_impl(id, client, service, info, recv),
            subscribe_service,
        )
    }

    pub fn remove(&mut self, proxy: ProxyId) -> Option<(ServiceCookie, HashSet<u32>, bool)> {
        let entry = self.entries.remove(&proxy)?;
        let (cookie, mut events) = entry.remove();

        let Entry::Occupied(mut entries) = self.services.entry(cookie) else {
            panic!("inconsistent state");
        };

        let contained = entries.get_mut().remove(&proxy);
        debug_assert!(contained);

        let unsubscribe_service = if entries.get().is_empty() {
            entries.remove();
            true
        } else {
            events.retain(|&event| {
                entries.get().iter().any(|proxy| {
                    self.entries
                        .get(proxy)
                        .expect("inconsistent state")
                        .is_subscribed_to(event)
                })
            });

            false
        };

        Some((cookie, events, unsubscribe_service))
    }

    pub fn remove_service(&mut self, service: ServiceCookie) {
        if let Some(proxies) = self.services.remove(&service) {
            for proxy in proxies {
                let entry = self.entries.remove(&proxy);
                debug_assert!(entry.is_some());
            }
        }
    }

    pub fn subscribe(&mut self, proxy: ProxyId, event: u32) -> SubscribeResult {
        let Some(entry) = self.entries.get_mut(&proxy) else {
            return SubscribeResult::InvalidProxy;
        };

        let service = entry.service();

        if entry.subscribe(event)
            && !self
                .entries
                .iter()
                .any(|(&id, entry)| (id != proxy) && entry.is_subscribed_to(event))
        {
            SubscribeResult::Forward(service)
        } else {
            SubscribeResult::Noop
        }
    }

    pub fn unsubscribe(&mut self, proxy: ProxyId, event: u32) -> SubscribeResult {
        let Some(entry) = self.entries.get_mut(&proxy) else {
            return SubscribeResult::InvalidProxy;
        };

        let service = entry.service();

        if entry.unsubscribe(event)
            && !self
                .entries
                .values()
                .any(|entry| entry.is_subscribed_to(event))
        {
            SubscribeResult::Forward(service)
        } else {
            SubscribeResult::Noop
        }
    }

    pub fn emit(&self, service: ServiceCookie, event: u32, args: SerializedValue) {
        if let Some(proxies) = self.services.get(&service) {
            let mut proxies = proxies.iter().peekable();

            while let Some(proxy) = proxies.next() {
                let proxy = self.entries.get(proxy).expect("inconsistent state");

                if proxy.is_subscribed_to(event) {
                    // Avoid cloning args for the last proxy.
                    if proxies.peek().is_some() {
                        proxy.emit(event, args.clone());
                    } else {
                        proxy.emit(event, args);
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct ProxyEntry {
    service: ServiceCookie,
    send: UnboundedSender<Event>,
    events: HashSet<u32>,
}

impl ProxyEntry {
    fn new(service: ServiceCookie, send: UnboundedSender<Event>) -> Self {
        Self {
            service,
            send,
            events: HashSet::new(),
        }
    }

    fn service(&self) -> ServiceCookie {
        self.service
    }

    fn remove(self) -> (ServiceCookie, HashSet<u32>) {
        (self.service, self.events)
    }

    fn subscribe(&mut self, event: u32) -> bool {
        self.events.insert(event)
    }

    fn unsubscribe(&mut self, event: u32) -> bool {
        self.events.remove(&event)
    }

    fn is_subscribed_to(&self, event: u32) -> bool {
        self.events.contains(&event)
    }

    fn emit(&self, event: u32, args: SerializedValue) {
        debug_assert!(self.events.contains(&event));
        let _ = self.send.unbounded_send(Event::new(event, args));
    }
}

#[derive(Debug)]
pub(crate) enum SubscribeResult {
    Forward(ServiceCookie),
    Noop,
    InvalidProxy,
}
