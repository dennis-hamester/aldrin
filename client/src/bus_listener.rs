use crate::error::Error;
use crate::handle::Handle;
use aldrin_proto::{BusEvent, BusListenerFilter, BusListenerScope};
use futures_core::stream::{FusedStream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

/// TODO
#[derive(Debug)]
pub struct BusListener {}

impl BusListener {
    /// TODO
    pub fn handle(&self) -> &Handle {
        todo!()
    }

    /// TODO
    pub async fn destroy(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub fn add_filter(&mut self, _filter: BusListenerFilter) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub fn remove_filter(&mut self, _filter: BusListenerFilter) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub fn clear_filters(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub fn filters(&self) -> impl Iterator<Item = BusListenerFilter> + '_ {
        None.into_iter()
    }

    /// TODO
    pub fn has_filter(&self, _filter: BusListenerFilter) -> bool {
        todo!()
    }

    /// TODO
    pub fn has_any_filters(&self) -> bool {
        todo!()
    }

    /// TODO
    pub fn has_no_filters(&self) -> bool {
        todo!()
    }

    /// TODO
    pub fn num_filters(&self) -> usize {
        todo!()
    }

    /// TODO
    pub async fn start(&mut self, _scope: BusListenerScope) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub async fn stop(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// TODO
    pub fn scope(&self) -> Option<BusListenerScope> {
        todo!()
    }

    /// TODO
    pub fn finished(&self) -> bool {
        todo!()
    }

    /// TODO
    pub fn poll_next_event(&mut self, _cx: &mut Context) -> Poll<Option<BusEvent>> {
        todo!()
    }

    /// TODO
    pub async fn next_event(&mut self) -> Option<BusEvent> {
        todo!()
    }
}

impl Stream for BusListener {
    type Item = BusEvent;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<BusEvent>> {
        todo!()
    }
}

impl FusedStream for BusListener {
    fn is_terminated(&self) -> bool {
        todo!()
    }
}
