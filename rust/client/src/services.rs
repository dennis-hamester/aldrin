use super::ServiceId;
use futures_channel::mpsc;
use futures_core::stream::{FusedStream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

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

#[derive(Debug, Copy, Clone)]
pub enum ServiceEvent {
    Created(ServiceId),
    Destroyed(ServiceId),
}
