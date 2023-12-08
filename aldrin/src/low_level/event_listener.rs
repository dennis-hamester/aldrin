#[cfg(test)]
mod test;

use crate::core::{SerializedValue, ServiceCookie, ServiceId};
use crate::handle::Handle;
use crate::low_level::Event;
use crate::Error;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_core::stream::{FusedStream, Stream};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

type Subscriptions = (ServiceId, HashSet<u32>);

/// Listens to events from services.
///
/// Events can be [`subscribe`d](EventListener::subscribe) and
/// [`unsubscribe`d](EventListener::unsubscribe). After subscribing to events, this type should be
/// polled through its implementation of [`Stream`].
///
/// Subscriptions can be removed implicitly, e.g. when a [`Service`](crate::Service) has been
/// destroyed.  When there are no subscriptions left (or when none have been made in the first
/// place) [`Stream::poll_next`] will return `None`.
///
/// When the [`Client`](crate::Client) shuts down, all subscriptions are removed and
/// [`Stream::poll_next`] will return `None` as well.
///
/// [`EventListener`] holds an internal [`Handle`] and will thus prevent the
/// [`Client`](crate::Client) from shutting down automatically. The [`Handle`] is only released when
/// [`EventListener`] is dropped.
///
/// This is low-level type. You should generally use the auto-generated event streams instead, which
/// do not require knowledge of event ids and provide better ergonomics for event arguments.
///
/// # Examples
///
/// ```
/// use aldrin::low_level::Event;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio::TestBroker::new();
/// # let client = broker.add_client().await;
/// # let obj = client.create_object(aldrin::core::ObjectUuid::new_v4()).await?;
/// # let mut svc = obj.create_service(aldrin::core::ServiceUuid::new_v4(), 0).await?;
/// # let service_id = svc.id();
/// let mut event_listener = client.create_event_listener();
///
/// event_listener.subscribe(service_id, 1).await?;
/// event_listener.subscribe(service_id, 2).await?;
///
/// # client.emit_event(service_id, 1, &32u32)?;
/// while let Some(event) = event_listener.next_event().await {
///     match event {
///         Event { id: 1, value, .. } => {
///             let value: u32 = value.deserialize()?;
///             println!("Event 1 with u32 value {value}.");
///             # client.emit_event(service_id, 2, "Hello, world!")?;
///         }
///
///         Event { id: 2, value, .. } => {
///             let value: String = value.deserialize()?;
///             println!("Event 2 with string value {value}.");
///             # svc.destroy().await?;
///         }
///
///         _ => unreachable!(),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[must_use = "streams do nothing unless you poll them"]
pub struct EventListener {
    id: EventListenerId,
    client: Handle,
    recv: UnboundedReceiver<EventListenerRequest>,
    send: UnboundedSender<EventListenerRequest>,
    subscriptions: HashMap<ServiceCookie, Subscriptions>,
}

impl EventListener {
    /// Creates a new [`EventListener`].
    pub fn new(client: Handle) -> Self {
        let (send, recv) = unbounded();

        Self {
            id: EventListenerId::new(),
            client,
            recv,
            send,
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribes to an event.
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
    /// This function returns `true`, if the event `id` of service `service_id` was subscribed to
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

    /// Polls for the next event.
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<Event>> {
        loop {
            if self.subscriptions.is_empty() {
                return Poll::Ready(None);
            }

            match Pin::new(&mut self.recv).poll_next(cx) {
                Poll::Ready(Some(EventListenerRequest::EmitEvent(service_cookie, event, args))) => {
                    if let Some(service_id) = self
                        .subscriptions
                        .get(&service_cookie)
                        .and_then(|(id, s)| if s.contains(&event) { Some(*id) } else { None })
                    {
                        return Poll::Ready(Some(Event::new(service_id, event, args)));
                    }
                }

                Poll::Ready(Some(EventListenerRequest::ServiceDestroyed(service_cookie))) => {
                    self.subscriptions.remove(&service_cookie);
                }

                Poll::Ready(None) => {
                    self.subscriptions.clear();
                    return Poll::Ready(None);
                }

                Poll::Pending => return Poll::Pending,
            }
        }
    }

    /// Returns the next event.
    pub async fn next_event(&mut self) -> Option<Event> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }
}

impl Stream for EventListener {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Event>> {
        self.poll_next_event(cx)
    }
}

impl FusedStream for EventListener {
    fn is_terminated(&self) -> bool {
        self.recv.is_terminated()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EventListenerId(Uuid);

impl EventListenerId {
    pub fn new() -> Self {
        EventListenerId(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub(crate) enum EventListenerRequest {
    EmitEvent(ServiceCookie, u32, SerializedValue),
    ServiceDestroyed(ServiceCookie),
}
