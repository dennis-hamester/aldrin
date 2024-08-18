use aldrin_core::ServiceCookie;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct BrokerSubscriptions {
    entries: HashMap<ServiceCookie, Service>,
}

impl BrokerSubscriptions {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, service: ServiceCookie, event: u32) {
        self.entries.entry(service).or_default().subscribe(event);
    }

    pub fn unsubscribe(&mut self, service: ServiceCookie, event: u32) {
        if let Entry::Occupied(mut entry) = self.entries.entry(service) {
            entry.get_mut().unsubscribe(event);

            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    pub fn subscribe_all(&mut self, service: ServiceCookie) {
        self.entries.entry(service).or_default().subscribe_all();
    }

    pub fn unsubscribe_all(&mut self, service: ServiceCookie) {
        if let Entry::Occupied(mut entry) = self.entries.entry(service) {
            entry.get_mut().unsubscribe_all();

            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    pub fn emit(&self, service: ServiceCookie, event: u32) -> bool {
        self.entries
            .get(&service)
            .map(|entry| entry.emit(event))
            .unwrap_or(false)
    }

    pub fn remove_service(&mut self, service: ServiceCookie) {
        self.entries.remove(&service);
    }
}

#[derive(Debug, Default)]
struct Service {
    events: HashSet<u32>,
    all_events: bool,
}

impl Service {
    fn is_empty(&self) -> bool {
        !self.all_events && self.events.is_empty()
    }

    fn subscribe(&mut self, event: u32) {
        self.events.insert(event);
    }

    fn unsubscribe(&mut self, event: u32) {
        self.events.remove(&event);
    }

    fn subscribe_all(&mut self) {
        self.all_events = true;
    }

    fn unsubscribe_all(&mut self) {
        self.all_events = false;
    }

    fn emit(&self, event: u32) -> bool {
        self.all_events || self.events.contains(&event)
    }
}
