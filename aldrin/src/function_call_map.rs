use crate::error::Error;
use crate::serial_map::SerialMap;
use aldrin_core::message::CallFunctionResult;
use futures_channel::oneshot::Sender;
use std::task::{Context, Poll};
use std::time::Instant;

type ResultSender = Sender<Result<(CallFunctionResult, Instant), Error>>;

#[derive(Debug)]
pub struct FunctionCallMap {
    inner: SerialMap<State>,
}

impl FunctionCallMap {
    pub fn new() -> Self {
        Self {
            inner: SerialMap::new(),
        }
    }

    pub fn insert(&mut self, sender: ResultSender) -> u32 {
        self.inner.insert(State::Pending(sender))
    }

    pub fn remove(&mut self, serial: u32) -> Option<ResultSender> {
        match self.inner.remove(serial)? {
            State::Pending(sender) => Some(sender),
            State::Aborted => None,
        }
    }

    pub fn abort(&mut self, serial: u32) {
        if let Some(state) = self.inner.get_mut(serial) {
            *state = State::Aborted;
        }
    }

    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<u32> {
        for (serial, state) in self.inner.iter_mut() {
            if let State::Pending(sender) = state {
                if sender.poll_canceled(cx) == Poll::Ready(()) {
                    return Poll::Ready(serial);
                }
            }
        }

        Poll::Pending
    }
}

#[derive(Debug)]
enum State {
    Pending(ResultSender),
    Aborted,
}
