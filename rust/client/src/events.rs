use super::{Error, Handle, ServiceCookie, ServiceId};
use aldrin_proto::Value;
use futures_channel::mpsc::{channel, Receiver, Sender};
use futures_core::stream::Stream;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

type Subscriptions = (ServiceId, HashSet<u32>);

#[derive(Debug)]
pub struct Events {
    id: EventsId,
    client: Handle,
    recv: Receiver<EventsRequest>,
    send: Sender<EventsRequest>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
}

impl Events {
    pub(crate) fn new(client: Handle, fifo_size: usize) -> Self {
        let (send, recv) = channel(fifo_size);
        Events {
            id: EventsId::new(),
            client,
            recv,
            send,
            subscriptions: HashMap::new(),
        }
    }

    pub async fn subscribe(&mut self, service_id: ServiceId, id: u32) -> Result<bool, Error> {
        match self.subscriptions.entry(service_id.cookie) {
            Entry::Vacant(entry) => self
                .client
                .subscribe_event(self.id, service_id, id, self.send.clone())
                .await
                .map(|()| {
                    let (_, events) = entry.insert((service_id, HashSet::with_capacity(1)));
                    events.insert(id);
                    true
                }),

            Entry::Occupied(mut entry) => {
                if entry.get_mut().1.contains(&id) {
                    Ok(false)
                } else {
                    self.client
                        .subscribe_event(self.id, service_id, id, self.send.clone())
                        .await
                        .map(|()| {
                            entry.get_mut().1.insert(id);
                            true
                        })
                }
            }
        }
    }

    pub async fn unsubscribe(&mut self, service_id: ServiceId, id: u32) -> Result<bool, Error> {
        match self.subscriptions.entry(service_id.cookie) {
            Entry::Occupied(mut entry) => {
                debug_assert_eq!(entry.get().0, service_id);
                if entry.get().1.contains(&id) {
                    self.client
                        .unsubscribe_event(self.id, service_id, id)
                        .await
                        .map(move |()| {
                            entry.get_mut().1.remove(&id);
                            if entry.get().1.is_empty() {
                                entry.remove();
                            }
                            true
                        })
                } else {
                    Ok(false)
                }
            }

            Entry::Vacant(_) => Ok(false),
        }
    }
}

impl Stream for Events {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Event>> {
        loop {
            if self.subscriptions.is_empty() {
                return Poll::Ready(None);
            }

            match Pin::new(&mut self.recv).poll_next(cx) {
                Poll::Ready(Some(EventsRequest::EmitEvent(service_cookie, event, args))) => {
                    if let Some(service_id) = self
                        .subscriptions
                        .get(&service_cookie)
                        .and_then(|(id, s)| if s.contains(&event) { Some(*id) } else { None })
                    {
                        return Poll::Ready(Some(Event::new(service_id, event, args)));
                    }
                }

                Poll::Ready(Some(EventsRequest::ServiceDestroyed(service_cookie))) => {
                    self.subscriptions.remove(&service_cookie);
                }

                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EventsId(Uuid);

impl EventsId {
    pub fn new() -> Self {
        EventsId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub service_id: ServiceId,
    pub id: u32,
    pub args: Value,
}

impl Event {
    pub(crate) fn new(service_id: ServiceId, id: u32, args: Value) -> Self {
        Event {
            service_id,
            id,
            args,
        }
    }
}

#[derive(Debug)]
pub(crate) enum EventsRequest {
    EmitEvent(ServiceCookie, u32, Value),
    ServiceDestroyed(ServiceCookie),
}
