use crate::handle::CloseChannelEndFuture;
use crate::{Error, Handle};
use aldrin_core::{ChannelCookie, ChannelEnd, SerializedValue};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{future, mem};

#[derive(Debug)]
pub(crate) struct RawChannel<const SENDER: bool> {
    client: Handle,
    cookie: ChannelCookie,
    claimed: bool,
    state: State,
}

impl<const SENDER: bool> RawChannel<SENDER> {
    fn new(client: Handle, cookie: ChannelCookie, claimed: bool) -> Self {
        Self {
            client,
            cookie,
            claimed,
            state: State::Open,
        }
    }

    pub(crate) fn unclaimed(client: Handle, cookie: ChannelCookie) -> Self {
        Self::new(client, cookie, false)
    }

    pub(crate) fn claimed(client: Handle, cookie: ChannelCookie) -> Self {
        Self::new(client, cookie, true)
    }

    pub(crate) fn client(&self) -> &Handle {
        &self.client
    }

    pub(crate) fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    pub(crate) fn is_open(&self) -> bool {
        self.state.is_open()
    }

    pub(crate) fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        let mut closing = match mem::replace(&mut self.state, State::Closed) {
            State::Open => self.begin_close()?,
            State::Closing(closing) => closing,
            State::Closed => return Poll::Ready(Ok(())),
        };

        match Pin::new(&mut closing).poll(cx) {
            Poll::Ready(res) => Poll::Ready(res),

            Poll::Pending => {
                self.state = State::Closing(closing);
                Poll::Pending
            }
        }
    }

    pub(crate) async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    pub(crate) fn unbind(mut self) -> ChannelCookie {
        debug_assert!(!self.claimed);
        self.state = State::Closed;
        self.cookie
    }

    pub(crate) fn set_claimed(&mut self) {
        debug_assert!(!self.claimed);
        self.claimed = true;
    }

    fn begin_close(&mut self) -> Result<CloseChannelEndFuture, Error> {
        self.client
            .close_channel_end(self.cookie, Self::channel_end(), self.claimed)
    }

    const fn channel_end() -> ChannelEnd {
        if SENDER {
            ChannelEnd::Sender
        } else {
            ChannelEnd::Receiver
        }
    }
}

impl RawChannel<true> {
    pub(crate) fn send_item(&self, item: SerializedValue) -> Result<(), Error> {
        if self.is_open() {
            self.client.send_item(self.cookie, item)
        } else {
            Err(Error::InvalidChannel)
        }
    }
}

impl RawChannel<false> {
    pub(crate) fn add_channel_capacity(&self, capacity: u32) {
        if self.is_open() {
            let _ = self.client.add_channel_capacity(self.cookie, capacity);
        }
    }
}

impl<const SENDER: bool> Drop for RawChannel<SENDER> {
    fn drop(&mut self) {
        if self.is_open() {
            let _ = self.begin_close();
        }
    }
}

#[derive(Debug)]
enum State {
    Open,
    Closing(CloseChannelEndFuture),
    Closed,
}

impl State {
    fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}
