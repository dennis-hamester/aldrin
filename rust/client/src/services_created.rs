use super::ServiceId;
use futures_channel::mpsc;
use futures_core::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct ServicesCreated(mpsc::Receiver<ServiceId>);

impl ServicesCreated {
    pub(crate) fn new(events: mpsc::Receiver<ServiceId>) -> Self {
        ServicesCreated(events)
    }
}

impl Stream for ServicesCreated {
    type Item = ServiceId;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<ServiceId>> {
        let events = Pin::new(&mut Pin::into_inner(self).0);
        events.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
