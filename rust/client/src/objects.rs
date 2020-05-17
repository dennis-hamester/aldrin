use super::ObjectId;
use futures_channel::mpsc;
use futures_core::stream::{FusedStream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
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

#[derive(Debug, Copy, Clone)]
pub enum ObjectEvent {
    Created(ObjectId),
    Destroyed(ObjectId),
}
