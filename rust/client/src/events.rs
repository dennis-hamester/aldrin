use super::{Error, Handle, ServiceCookie, ServiceId};
use aldrin_proto::Value;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_core::stream::{FusedStream, Stream};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

type Subscriptions = (ServiceId, HashSet<u32>);

/// Manages subscriptions to service events.
///
/// This struct is created by [`Handle::events`]. Events can be [`subscribe`d](Events::subscribe)
/// and [`unsubscribe`d](Events::unsubscribe).
///
/// After subscribing to events, this struct should be polled through its implementation of
/// [`Stream`].
///
/// Subscriptions can removed implicitly, e.g. when a service has been destroyed of which events
/// were subscribed. When there are no subscriptions left (or when none have been made in the first
/// place) [`Stream::poll_next`] will return [`None`].
///
/// When the client shuts down, [`Stream::poll_next`] will return [`None`] as well.
///
/// # Examples
///
/// ```ignore
/// // For StreamExt::next()
/// use futures::stream::StreamExt;
///
/// let mut events = handle.events();
///
/// // Subscribe to a few events.
/// events.subscribe(svc_id, 1).await?;
/// events.subscribe(svc_id, 2).await?;
///
/// // Handle next event.
/// if let Some(ev) = events.next().await {
///     match ev.id {
///         1 => { ... }
///         2 => { ... }
///         _ => unreachable!(),
///     }
/// }
/// ```
#[derive(Debug)]
#[must_use = "streams do nothing unless you poll them"]
pub struct Events {
    id: EventsId,
    client: Handle,
    recv: UnboundedReceiver<EventsRequest>,
    send: UnboundedSender<EventsRequest>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
}

impl Events {
    pub(crate) fn new(client: Handle) -> Self {
        let (send, recv) = unbounded();
        Events {
            id: EventsId::new(),
            client,
            recv,
            send,
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribe to an event.
    ///
    /// This function returns `true`, if the event `id` of service `service_id` was not already
    /// subscribed to. Otherwise `false` is returned.
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

    /// Unsubscribe from an event.
    ///
    /// This function return `true`, if the event `id` of service `service_id` was subscribed to
    /// before the call to this function and is now unsubscribed from. Otherwise `false` is
    /// returned.
    pub fn unsubscribe(&mut self, service_id: ServiceId, id: u32) -> Result<bool, Error> {
        match self.subscriptions.entry(service_id.cookie) {
            Entry::Occupied(mut entry) => {
                debug_assert_eq!(entry.get().0, service_id);
                if entry.get().1.contains(&id) {
                    self.client
                        .unsubscribe_event(self.id, service_id, id)
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

impl FusedStream for Events {
    fn is_terminated(&self) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EventsId(Uuid);

impl EventsId {
    pub fn new() -> Self {
        EventsId(Uuid::new_v4())
    }
}

/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event {
    /// Id of the service which emitted the event.
    pub service_id: ServiceId,

    /// Id of the event.
    pub id: u32,

    /// Arguments of the event.
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
