use aldrin_proto::ObjectId;
use futures_channel::mpsc;
use futures_core::stream::{FusedStream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream of object creation and destruction events.
///
/// [`Objects`] is created with [`Handle::objects`](crate::Handle::objects) and can be used to
/// discover and track [`Object`s](crate::Object) on the bus.
///
/// If [`Objects`] is created with
/// [`SubscribeMode::CurrentOnly`](crate::SubscribeMode::CurrentOnly), then the stream will
/// automatically end (return `None`) after it has returned [`ObjectId`s](ObjectId) for all current
/// [`Object`s](crate::Object) on the bus.
///
/// If using either [`SubscribeMode::NewOnly`](crate::SubscribeMode::NewOnly) or
/// [`SubscribeMode::All`](crate::SubscribeMode::All), then the stream will end only when the
/// [`Client`](crate::Client) is shut down.
///
/// # Examples
/// ```
/// use aldrin_client::{ObjectEvent, SubscribeMode};
/// use futures::stream::StreamExt;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// let mut objects = handle.objects(SubscribeMode::CurrentOnly)?;
/// # handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
///
/// while let Some(event) = objects.next().await {
///     match event {
///         ObjectEvent::Created(object_id) => {
///             println!("Object created: {:?}.", object_id);
///         }
///
///         ObjectEvent::Destroyed(object_id) => {
///             println!("Object destroyed: {:?}.", object_id);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`Objects`] can be used to easily search for a particular [`Object`](crate::Object) and acquire
/// its [`ObjectId`]:
///
/// ```
/// use aldrin_client::{ObjectEvent, ObjectUuid, SubscribeMode};
/// use futures::future;
/// use futures::stream::StreamExt;
///
/// // d434e9c8-6230-4fa6-b61c-1babbaa37a4f
/// const INTERESTING_OBJECT_UUID: ObjectUuid =
///     ObjectUuid::from_u128(0xd434e9c862304fa6b61c1babbaa37a4f);
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_test::tokio_based::TestBroker::new();
/// # let handle = broker.add_client().await;
/// # let obj = handle.create_object(INTERESTING_OBJECT_UUID).await?;
/// let mut objects = handle.objects(SubscribeMode::CurrentOnly)?;
///
/// let object_id = objects
///     .filter_map(|event| {
///         future::ready(match event {
///             ObjectEvent::Created(object_id) if object_id.uuid == INTERESTING_OBJECT_UUID => {
///                 Some(object_id)
///             }
///             _ => None,
///         })
///     })
///     .next()
///     .await;
///
/// # assert!(object_id.is_some());
/// if let Some(object_id) = object_id {
///     # assert_eq!(object_id.uuid, INTERESTING_OBJECT_UUID);
///     println!("Object {} found.", INTERESTING_OBJECT_UUID);
///     // Do something with object_id here ...
/// } else {
///     println!("Object {} not found.", INTERESTING_OBJECT_UUID);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[must_use = "streams do nothing unless you poll them"]
pub struct Objects(mpsc::UnboundedReceiver<ObjectEvent>);

impl Objects {
    pub(crate) fn new(events: mpsc::UnboundedReceiver<ObjectEvent>) -> Self {
        Objects(events)
    }
}

impl Stream for Objects {
    type Item = ObjectEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<ObjectEvent>> {
        Pin::new(&mut Pin::into_inner(self).0).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedStream for Objects {
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

/// Event about a created or destroyed object.
///
/// This is the element type of the [`Objects`] stream. See that type for more information and usage
/// examples.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectEvent {
    /// An object with the specified id was created.
    Created(ObjectId),

    /// An object with the specified id was destroyed.
    Destroyed(ObjectId),
}
