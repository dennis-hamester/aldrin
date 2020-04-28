use super::ObjectId;
use futures_channel::mpsc;
use futures_core::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct ObjectsCreated(mpsc::UnboundedReceiver<ObjectId>);

impl ObjectsCreated {
    pub(crate) fn new(events: mpsc::UnboundedReceiver<ObjectId>) -> Self {
        ObjectsCreated(events)
    }
}

impl Stream for ObjectsCreated {
    type Item = ObjectId;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<ObjectId>> {
        let events = Pin::new(&mut Pin::into_inner(self).0);
        events.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
