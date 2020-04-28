use super::ServiceId;
use futures_channel::mpsc;
use futures_core::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct ServicesDestroyed(mpsc::UnboundedReceiver<ServiceId>);

impl ServicesDestroyed {
    pub(crate) fn new(events: mpsc::UnboundedReceiver<ServiceId>) -> Self {
        ServicesDestroyed(events)
    }
}

impl Stream for ServicesDestroyed {
    type Item = ServiceId;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<ServiceId>> {
        let events = Pin::new(&mut Pin::into_inner(self).0);
        events.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
