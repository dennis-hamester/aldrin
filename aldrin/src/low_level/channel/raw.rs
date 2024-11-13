use crate::core::{ChannelCookie, ChannelEnd, SerializedValue};
use crate::error::Error;
use crate::handle::{CloseChannelEndFuture, Handle};
use std::future::{self, Future};
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

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

    pub fn unclaimed(client: Handle, cookie: ChannelCookie) -> Self {
        Self::new(client, cookie, false)
    }

    pub fn claimed(client: Handle, cookie: ChannelCookie) -> Self {
        Self::new(client, cookie, true)
    }

    pub fn client(&self) -> &Handle {
        &self.client
    }

    pub fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    pub fn is_open(&self) -> bool {
        self.state.is_open()
    }

    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
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

    pub async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    pub fn unbind(mut self) -> ChannelCookie {
        debug_assert!(!self.claimed);
        self.state = State::Closed;
        self.cookie
    }

    pub fn set_claimed(&mut self) {
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
    pub fn send_item(&self, item: SerializedValue) -> Result<(), Error> {
        if self.is_open() {
            self.client.send_item(self.cookie, item)
        } else {
            Err(Error::InvalidChannel)
        }
    }
}

impl RawChannel<false> {
    pub fn add_channel_capacity(&self, capacity: u32) {
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
        matches!(self, State::Open)
    }
}
