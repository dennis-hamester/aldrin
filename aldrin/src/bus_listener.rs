#[cfg(test)]
mod test;

use crate::error::Error;
use crate::handle::Handle;
use aldrin_core::{BusEvent, BusListenerCookie, BusListenerFilter, BusListenerScope};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_core::stream::{FusedStream, Stream};
use std::collections::HashSet;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Monitors the bus for the creation and destruction of objects and services.
///
/// `BusListener`s use [`BusListenerFilter`] to specify which objects and/or services to
/// consider. It is possible to register interest in either a set of specific objects (based on
/// their [`ObjectUuid`](crate::core::ObjectUuid)), any object or no objects at all. The same
/// applies also to services. In all cases, filters match both creation and destruction events. It
/// is not possible to limit a `BusListener` to just one type.
///
/// `BusListener`s must be started before they emit events. At this point a [`BusListenerScope`] has
/// to be specified, which controls whether to consider current objects/services, future ones, or
/// both.
///
/// It is also possible to repeatedly start and stop a `BusListener`, as well as add and remove
/// filters at any time. However, some caveats need to be kept in mind:
///
/// - Adding filters will not cause events to be emitted for objects/services which exist already on
///   the bus (if the `BusListener` is started and the scope matches). For this to work, it must be
///   stopped and restarted.
/// - Adding and removing filters is not synchronized with the broker (the respective functions are
///   not `async`). This means that they will not take effect immediately.
///
/// # Examples
///
/// ## Enumerating all current objects and services
///
/// ```
/// use aldrin::core::{BusEvent, BusListenerFilter, BusListenerScope, ObjectUuid, ServiceUuid};
/// use aldrin::low_level::ServiceInfo;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut broker = aldrin_test::tokio::TestBroker::new();
/// # let handle = broker.add_client().await;
///
/// // Create a few objects and services.
/// let info = ServiceInfo::new(0);
/// let obj1 = handle.create_object(ObjectUuid::new_v4()).await?;
/// let service1 = obj1.create_service(ServiceUuid::new_v4(), info).await?;
/// let obj2 = handle.create_object(ObjectUuid::new_v4()).await?;
/// let service2 = obj2.create_service(ServiceUuid::new_v4(), info).await?;
///
/// // Create a bus listener.
/// let mut bus_listener = handle.create_bus_listener().await?;
///
/// // Add filters for all objects and services.
/// bus_listener.add_filter(BusListenerFilter::any_object())?;
/// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
///
/// // Start the bus listener and limit the scope to only current objects and services.
/// bus_listener.start(BusListenerScope::Current).await?;
///
/// // Drain the bus listener of all events. When using [`BusListenerScope::Current`], then only
/// // creation events will be emitted and `None` will be returned when the bus listener finishes.
/// # let mut num = 0;
/// while let Some(event) = bus_listener.next_event().await {
///     match event {
///         BusEvent::ObjectCreated(id) => {
///             println!("Object {} found.", id.uuid);
///         }
///
///         BusEvent::ServiceCreated(id) => {
///             println!("Service {} on object {} found.", id.uuid, id.object_id.uuid);
///         }
///
///         BusEvent::ObjectDestroyed(_) | BusEvent::ServiceDestroyed(_) => unreachable!(),
///     }
///     # num += 1;
/// }
/// # assert_eq!(num, 4);
/// # assert!(bus_listener.is_finished());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct BusListener {
    cookie: BusListenerCookie,
    client: Handle,
    filters: HashSet<BusListenerFilter>,
    scope: Option<BusListenerScope>,
    pending_started: usize,
    pending_stopped: usize,
    pending_current_finished: usize,
    events: UnboundedReceiver<BusListenerEvent>,
}

impl BusListener {
    /// Creates a new bus listener.
    pub async fn new(client: &Handle) -> Result<Self, Error> {
        client.create_bus_listener().await
    }

    pub(crate) fn new_impl(
        cookie: BusListenerCookie,
        client: Handle,
        events: UnboundedReceiver<BusListenerEvent>,
    ) -> Self {
        Self {
            cookie,
            client,
            filters: HashSet::new(),
            scope: None,
            pending_started: 0,
            pending_stopped: 0,
            pending_current_finished: 0,
            events,
        }
    }

    /// Returns a handle to the client that was used to create the bus listener.
    pub fn client(&self) -> &Handle {
        &self.client
    }

    /// Destroys the bus listener.
    ///
    /// Bus listener are also destroyed implicitly when they fall out of scope.
    ///
    /// Events that have already been emitted by the broker may still be emitted by the bus listener
    /// after destroying it.
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.client.destroy_bus_listener(self.cookie).await?;

        self.events.close();
        self.pending_started = 0;
        self.pending_stopped = 0;
        self.pending_current_finished = 0;

        Ok(())
    }

    /// Adds a filter to the bus listener.
    ///
    /// For an event to be emitted, there must be a filter that matches it. Adding the same filter
    /// twice has no effect. In particular, there is no reference counting for filters.
    ///
    /// Note that new filters do not affect events which are already in the bus listener's internal
    /// queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::{BusListenerFilter, ObjectUuid};
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// const MY_OBJECT_UUID: ObjectUuid = ObjectUuid(uuid!("0228a930-e194-4b1a-bc3e-6a4defb32527"));
    ///
    /// // Add a filter for `MY_OBJECT_UUID`.
    /// bus_listener.add_filter(BusListenerFilter::object(MY_OBJECT_UUID))?;
    ///
    /// // Add a filter for all services of `MY_OBJECT_UUID`.
    /// bus_listener.add_filter(BusListenerFilter::specific_object_any_service(MY_OBJECT_UUID))?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_filter(&mut self, filter: BusListenerFilter) -> Result<(), Error> {
        if self.filters.insert(filter) {
            self.client.add_bus_listener_filter(self.cookie, filter)
        } else {
            Ok(())
        }
    }

    /// Remove a filter from the bus listener.
    ///
    /// Removing a filter, that isn't present in the bus listener, has no effect.
    ///
    /// Note that filters do not affect events which are already in the bus listener's internal
    /// queue. Thus, events that match only a single filter, may still be emitted after removing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::BusListenerFilter;
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// // Remove the filter that matches all services of any object.
    /// bus_listener.remove_filter(BusListenerFilter::any_object_any_service())?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_filter(&mut self, filter: BusListenerFilter) -> Result<(), Error> {
        if self.filters.remove(&filter) {
            self.client.remove_bus_listener_filter(self.cookie, filter)
        } else {
            Ok(())
        }
    }

    /// Clears the set of all filters.
    ///
    /// The same caveats as with [`remove_filter`](Self::remove_filter) apply.
    pub fn clear_filters(&mut self) -> Result<(), Error> {
        if self.filters.is_empty() {
            Ok(())
        } else {
            self.filters.clear();
            self.client.clear_bus_listener_filters(self.cookie)
        }
    }

    /// Returns an iterator of all filters.
    ///
    /// The order in which the filters are returned is unspecified.
    pub fn filters(&self) -> impl Iterator<Item = BusListenerFilter> {
        self.filters.iter().copied()
    }

    /// Checks if a specific filter is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::BusListenerFilter;
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(bus_listener.has_filter(BusListenerFilter::any_object_any_service()));
    ///
    /// bus_listener.remove_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(!bus_listener.has_filter(BusListenerFilter::any_object_any_service()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_filter(&self, filter: BusListenerFilter) -> bool {
        self.filters.contains(&filter)
    }

    /// Checks if the bus listener has any filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::BusListenerFilter;
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// assert!(!bus_listener.has_any_filters());
    ///
    /// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(bus_listener.has_any_filters());
    ///
    /// bus_listener.remove_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(!bus_listener.has_any_filters());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_any_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// Checks if the bus listener has no filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::BusListenerFilter;
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// assert!(bus_listener.has_no_filters());
    ///
    /// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(!bus_listener.has_no_filters());
    ///
    /// bus_listener.remove_filter(BusListenerFilter::any_object_any_service())?;
    /// assert!(bus_listener.has_no_filters());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_no_filters(&self) -> bool {
        self.filters.is_empty()
    }

    /// Returns the number of filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::core::BusListenerFilter;
    /// # use uuid::uuid;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let mut bus_listener = handle.create_bus_listener().await?;
    ///
    /// assert_eq!(bus_listener.num_filters(), 0);
    ///
    /// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    /// assert_eq!(bus_listener.num_filters(), 1);
    ///
    /// bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    /// assert_eq!(bus_listener.num_filters(), 1); // No duplicates
    ///
    /// bus_listener.remove_filter(BusListenerFilter::any_object_any_service())?;
    /// assert_eq!(bus_listener.num_filters(), 0);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn num_filters(&self) -> usize {
        self.filters.len()
    }

    /// Starts the bus listener with the given scope.
    ///
    /// Bus listeners can only be started when they are stopped.
    pub async fn start(&mut self, scope: BusListenerScope) -> Result<(), Error> {
        self.client.start_bus_listener(self.cookie, scope).await?;
        self.pending_started += 1;

        if scope.includes_current() {
            self.pending_current_finished += 1;
        }

        Ok(())
    }

    /// Stops the bus listener.
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.client.stop_bus_listener(self.cookie).await?;
        self.pending_stopped += 1;
        Ok(())
    }

    /// Returns the bus listener's active scope, if any.
    ///
    /// The scope returned by this function does not change immediately after calling
    /// [`start`](Self::start). It returns the scope that is associated with the events that are
    /// currently returned by [`next_event`](Self::next_event) and
    /// [`poll_next_event`](Self::poll_next_event).
    pub fn scope(&self) -> Option<BusListenerScope> {
        self.scope
    }

    /// Indicates whether the bus listener can return more events.
    ///
    /// This function essentially indicates whether [`next_event`](Self::next_event) and
    /// [`poll_next_event`](Self::poll_next_event) can possible return `Some` or not. You should
    /// continue to call either function for as long as this function returns `false`.
    ///
    /// Bus listeners can only ever finish if they are stopped or if their scope is
    /// [`BusListenerScope::Current`] and all events have been emitted.
    pub fn is_finished(&self) -> bool {
        self.events.is_terminated()
            || (!self.includes_new()
                && (self.pending_started == 0)
                && (self.pending_stopped == 0)
                && (self.pending_current_finished == 0))
    }

    /// Polls the bus listener for an event.
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<BusEvent>> {
        loop {
            if self.is_finished() {
                break Poll::Ready(None);
            }

            match Pin::new(&mut self.events).poll_next(cx) {
                Poll::Ready(Some(BusListenerEvent::Started(scope))) => {
                    self.scope = Some(scope);
                    self.pending_started -= 1;
                }

                Poll::Ready(Some(BusListenerEvent::Stopped)) => {
                    self.scope = None;
                    self.pending_stopped -= 1;
                }

                Poll::Ready(Some(BusListenerEvent::Event(event))) => {
                    return Poll::Ready(Some(event));
                }

                Poll::Ready(Some(BusListenerEvent::CurrentFinished)) => {
                    self.pending_current_finished -= 1;
                }

                Poll::Ready(None) => break Poll::Ready(None),
                Poll::Pending => break Poll::Pending,
            }
        }
    }

    /// Await an event from the bus listener.
    pub async fn next_event(&mut self) -> Option<BusEvent> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }

    fn includes_new(&self) -> bool {
        self.scope.is_some_and(BusListenerScope::includes_new)
    }
}

impl Drop for BusListener {
    fn drop(&mut self) {
        self.client.destroy_bus_listener_now(self.cookie);
    }
}

impl Stream for BusListener {
    type Item = BusEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<BusEvent>> {
        self.poll_next_event(cx)
    }
}

impl FusedStream for BusListener {
    fn is_terminated(&self) -> bool {
        self.is_finished()
    }
}

#[derive(Debug)]
pub(crate) struct BusListenerHandle {
    filters: HashSet<BusListenerFilter>,
    events: UnboundedSender<BusListenerEvent>,
    scope: Option<BusListenerScope>,
    current_finished: bool,
}

impl BusListenerHandle {
    pub(crate) fn new(events: UnboundedSender<BusListenerEvent>) -> Self {
        Self {
            filters: HashSet::new(),
            events,
            scope: None,
            current_finished: false,
        }
    }

    pub(crate) fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
    }

    pub(crate) fn remove_filter(&mut self, filter: BusListenerFilter) {
        self.filters.remove(&filter);
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub(crate) fn start(&mut self, scope: BusListenerScope) -> bool {
        if self.scope.is_none() {
            self.scope = Some(scope);
            self.current_finished = !scope.includes_current();
            let _ = self.events.unbounded_send(BusListenerEvent::Started(scope));
            true
        } else {
            false
        }
    }

    pub(crate) fn stop(&mut self) -> bool {
        if self.scope.is_some() {
            self.scope = None;
            let _ = self.events.unbounded_send(BusListenerEvent::Stopped);
            true
        } else {
            false
        }
    }

    pub(crate) fn current_finished(&mut self) -> bool {
        if self.current_finished {
            false
        } else {
            let _ = self
                .events
                .unbounded_send(BusListenerEvent::CurrentFinished);
            self.current_finished = true;
            true
        }
    }

    pub(crate) fn emit_current(&self, event: BusEvent) -> bool {
        if self.includes_current() && !self.current_finished {
            let _ = self.events.unbounded_send(BusListenerEvent::Event(event));
            true
        } else {
            false
        }
    }

    pub(crate) fn emit_new_if_matches(&self, event: BusEvent) {
        if self.includes_new() && self.matches_filters(event) {
            let _ = self.events.unbounded_send(BusListenerEvent::Event(event));
        }
    }

    fn includes_current(&self) -> bool {
        self.scope.is_some_and(BusListenerScope::includes_current)
    }

    fn includes_new(&self) -> bool {
        self.scope.is_some_and(BusListenerScope::includes_new)
    }

    fn matches_filters(&self, event: BusEvent) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.matches_event(event))
    }
}

#[derive(Debug)]
pub(crate) enum BusListenerEvent {
    Started(BusListenerScope),
    Stopped,
    Event(BusEvent),
    CurrentFinished,
}
