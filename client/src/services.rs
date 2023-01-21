use aldrin_proto::ServiceId;
use futures_channel::mpsc;
use futures_core::stream::{FusedStream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream of service creation and destruction events.
///
/// [`Services`] is created with [`Handle::services`](crate::Handle::services) and can be used to
/// discover and track [`Service`s](crate::Service) on the bus.
///
/// If [`Services`] is created with
/// [`SubscribeMode::CurrentOnly`](crate::SubscribeMode::CurrentOnly), then the stream will
/// automatically end (return `None`) after it has returned [`ServiceId`s](ServiceId) for all
/// current [`Service`s](crate::Service) on the bus.
///
/// If using either [`SubscribeMode::NewOnly`](crate::SubscribeMode::NewOnly) or
/// [`SubscribeMode::All`](crate::SubscribeMode::All), then the stream will end only when the
/// [`Client`](crate::Client) is shut down.
///
/// # Examples
/// ```
/// use aldrin_client::{ServiceEvent, SubscribeMode};
/// use futures::stream::StreamExt;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// let mut services = handle.services(SubscribeMode::CurrentOnly)?;
/// # let mut object = handle.create_object(aldrin_proto::ObjectUuid::new_v4()).await?;
/// # object.create_service(aldrin_proto::ServiceUuid::new_v4(), 0).await?;
///
/// while let Some(event) = services.next().await {
///     match event {
///         ServiceEvent::Created(service_id) => {
///             println!("Service created: {:?}.", service_id);
///         }
///
///         ServiceEvent::Destroyed(service_id) => {
///             println!("Service destroyed: {:?}.", service_id);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`Services`] can be used to easily search for a particular [`Service`](crate::Service) and
/// acquire its [`ServiceId`]:
///
/// ```
/// use aldrin_client::{ServiceEvent, SubscribeMode};
/// use aldrin_proto::ServiceUuid;
/// use futures::future;
/// use futures::stream::StreamExt;
/// use uuid::uuid;
///
/// const INTERESTING_SERVICE_UUID: ServiceUuid =
///     ServiceUuid(uuid!("6d92452b-0cbc-493f-b16b-9f4ce2474a2e"));
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// # let obj = handle.create_object(aldrin_proto::ObjectUuid::new_v4()).await?;
/// # let svc = obj.create_service(INTERESTING_SERVICE_UUID, 0).await?;
/// let mut services = handle.services(SubscribeMode::CurrentOnly)?;
///
/// let service_id = services
///     .filter_map(|event| {
///         future::ready(match event {
///             ServiceEvent::Created(service_id) if service_id.uuid == INTERESTING_SERVICE_UUID => {
///                 Some(service_id)
///             }
///             _ => None,
///         })
///     })
///     .next()
///     .await;
///
/// # assert!(service_id.is_some());
/// if let Some(service_id) = service_id {
///     # assert_eq!(service_id.uuid, INTERESTING_SERVICE_UUID);
///     println!("Service {} found.", INTERESTING_SERVICE_UUID);
///     // Do something with service_id here ...
/// } else {
///     println!("Service {} not found.", INTERESTING_SERVICE_UUID);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[must_use = "streams do nothing unless you poll them"]
pub struct Services(mpsc::UnboundedReceiver<ServiceEvent>);

impl Services {
    pub(crate) fn new(events: mpsc::UnboundedReceiver<ServiceEvent>) -> Self {
        Services(events)
    }
}

impl Stream for Services {
    type Item = ServiceEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<ServiceEvent>> {
        Pin::new(&mut Pin::into_inner(self).0).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedStream for Services {
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

/// Event about a created or destroyed service.
///
/// This is the element type of the [`Services`] stream. See that type for more information and
/// usage examples.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServiceEvent {
    /// A service with the specified id was created.
    Created(ServiceId),

    /// A service with the specified id was destroyed.
    Destroyed(ServiceId),
}