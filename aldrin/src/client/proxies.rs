use crate::handle::Handle;
use crate::low_level::{Event, Proxy, ProxyId};
use aldrin_core::{SerializedValue, ServiceCookie, ServiceId, ServiceInfo};
use futures_channel::mpsc::{self, UnboundedSender};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::mem;
use std::time::Instant;

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

    pub fn remove(&mut self, proxy: ProxyId) -> Option<RemoveProxyResult> {
        let mut res = self.entries.remove(&proxy)?.remove();

        let Entry::Occupied(mut entries) = self.services.entry(res.service) else {
            panic!("inconsistent state");
        };

        let contained = entries.get_mut().remove(&proxy);
        debug_assert!(contained);

        if entries.get().is_empty() {
            entries.remove();
        } else {
            res.unsubscribe = false;

            res.events.retain(|&event| {
                entries.get().iter().all(|proxy| {
                    !self
                        .entries
                        .get(proxy)
                        .expect("inconsistent state")
                        .is_subscribed_to(event)
                })
            });

            res.all_events &= !self.is_any_subscribed_to_all(res.service, None);
        }

        Some(res)
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
        if entry.subscribe(event) && !self.is_any_subscribed_to(service, event, Some(proxy)) {
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
        if entry.unsubscribe(event) && !self.is_any_subscribed_to(service, event, None) {
            SubscribeResult::Forward(service)
        } else {
            SubscribeResult::Noop
        }
    }

    fn is_any_subscribed_to(
        &self,
        service: ServiceCookie,
        event: u32,
        except: Option<ProxyId>,
    ) -> bool {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.service() == service)
            .filter(|&(&id, _)| {
                if let Some(except) = except {
                    id != except
                } else {
                    true
                }
            })
            .any(|(_, entry)| entry.is_subscribed_to(event))
    }

    pub fn subscribe_all(&mut self, proxy: ProxyId) -> SubscribeResult {
        let Some(entry) = self.entries.get_mut(&proxy) else {
            return SubscribeResult::InvalidProxy;
        };

        let service = entry.service();
        if entry.subscribe_all() && !self.is_any_subscribed_to_all(service, Some(proxy)) {
            SubscribeResult::Forward(service)
        } else {
            SubscribeResult::Noop
        }
    }

    pub fn unsubscribe_all(&mut self, proxy: ProxyId) -> Option<UnsubscribeAllResult> {
        let mut res = self.entries.get_mut(&proxy)?.unsubscribe_all();
        let entries = self.services.get(&res.service).expect("inconsistent state");

        res.events.retain(|&event| {
            entries.iter().all(|proxy| {
                !self
                    .entries
                    .get(proxy)
                    .expect("inconsistent state")
                    .is_subscribed_to(event)
            })
        });

        res.all_events &= !self.is_any_subscribed_to_all(res.service, None);

        Some(res)
    }

    fn is_any_subscribed_to_all(&self, service: ServiceCookie, except: Option<ProxyId>) -> bool {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.service() == service)
            .filter(|&(&id, _)| {
                if let Some(except) = except {
                    id != except
                } else {
                    true
                }
            })
            .any(|(_, entry)| entry.is_subscribed_to_all())
    }

    pub fn emit(
        &self,
        service: ServiceCookie,
        event: u32,
        timestamp: Instant,
        args: SerializedValue,
    ) {
        if let Some(proxies) = self.services.get(&service) {
            let mut proxies = proxies.iter().peekable();

            while let Some(proxy) = proxies.next() {
                let proxy = self.entries.get(proxy).expect("inconsistent state");

                if proxy.is_subscribed_to_all() || proxy.is_subscribed_to(event) {
                    // Avoid cloning args for the last proxy.
                    if proxies.peek().is_some() {
                        proxy.emit(event, timestamp, args.clone());
                    } else {
                        proxy.emit(event, timestamp, args);
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
    all_events: bool,
}

impl ProxyEntry {
    fn new(service: ServiceCookie, send: UnboundedSender<Event>) -> Self {
        Self {
            service,
            send,
            events: HashSet::new(),
            all_events: false,
        }
    }

    fn service(&self) -> ServiceCookie {
        self.service
    }

    fn remove(self) -> RemoveProxyResult {
        RemoveProxyResult {
            service: self.service,
            unsubscribe: true,
            events: self.events,
            all_events: self.all_events,
        }
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

    fn subscribe_all(&mut self) -> bool {
        if self.all_events {
            false
        } else {
            self.all_events = true;
            true
        }
    }

    fn unsubscribe_all(&mut self) -> UnsubscribeAllResult {
        UnsubscribeAllResult {
            service: self.service,
            events: mem::take(&mut self.events),
            all_events: mem::take(&mut self.all_events),
        }
    }

    fn is_subscribed_to_all(&self) -> bool {
        self.all_events
    }

    fn emit(&self, event: u32, timestamp: Instant, args: SerializedValue) {
        debug_assert!(self.all_events || self.events.contains(&event));
        let _ = self.send.unbounded_send(Event::new(event, timestamp, args));
    }
}

#[derive(Debug)]
pub(crate) struct RemoveProxyResult {
    pub service: ServiceCookie,
    pub unsubscribe: bool,
    pub events: HashSet<u32>,
    pub all_events: bool,
}

#[derive(Debug)]
pub(crate) enum SubscribeResult {
    Forward(ServiceCookie),
    Noop,
    InvalidProxy,
}

#[derive(Debug)]
pub(crate) struct UnsubscribeAllResult {
    pub service: ServiceCookie,
    pub events: HashSet<u32>,
    pub all_events: bool,
}
