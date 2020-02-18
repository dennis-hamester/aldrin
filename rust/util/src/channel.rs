use aldrin_proto::{Message, Transport};
use futures_channel::mpsc::{self, SendError};
use futures_core::stream::Stream;
use futures_sink::Sink;
use std::error::Error;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn channel(fifo_size: usize) -> (ChannelTransport, ChannelTransport) {
    let (sender1, receiver1) = mpsc::channel(fifo_size);
    let (sender2, receiver2) = mpsc::channel(fifo_size);

    (
        ChannelTransport::new(None, receiver1, sender2),
        ChannelTransport::new(None, receiver2, sender1),
    )
}

pub fn channel_with_name<N>(fifo_size: usize, name: N) -> (ChannelTransport, ChannelTransport)
where
    N: Into<String>,
{
    let name = name.into();
    let (sender1, receiver1) = mpsc::channel(fifo_size);
    let (sender2, receiver2) = mpsc::channel(fifo_size);

    (
        ChannelTransport::new(Some(name.clone()), receiver1, sender2),
        ChannelTransport::new(Some(name), receiver2, sender1),
    )
}

#[derive(Debug)]
pub struct ChannelTransport {
    name: Option<String>,
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
}

impl ChannelTransport {
    fn new(
        name: Option<String>,
        receiver: mpsc::Receiver<Message>,
        sender: mpsc::Sender<Message>,
    ) -> Self {
        ChannelTransport {
            name,
            receiver,
            sender,
        }
    }
}

impl Stream for ChannelTransport {
    type Item = Result<Message, Closed>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Message, Closed>>> {
        Pin::new(&mut self.receiver)
            .poll_next(cx)
            .map(|msg| msg.map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.receiver.size_hint()
    }
}

impl Sink<Message> for ChannelTransport {
    type Error = Closed;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_ready(cx)
            .map(map_poll_send_error)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Closed> {
        Pin::new(&mut self.sender)
            .start_send(item)
            .map_err(map_send_error)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_flush(cx)
            .map(map_poll_send_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_close(cx)
            .map(map_poll_send_error)
    }
}

impl Transport for ChannelTransport {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

pub fn unbounded() -> (UnboundedTransport, UnboundedTransport) {
    let (sender1, receiver1) = mpsc::unbounded();
    let (sender2, receiver2) = mpsc::unbounded();

    (
        UnboundedTransport::new(None, receiver1, sender2),
        UnboundedTransport::new(None, receiver2, sender1),
    )
}

pub fn unbounded_with_name<N>(name: N) -> (UnboundedTransport, UnboundedTransport)
where
    N: Into<String>,
{
    let name = name.into();
    let (sender1, receiver1) = mpsc::unbounded();
    let (sender2, receiver2) = mpsc::unbounded();

    (
        UnboundedTransport::new(Some(name.clone()), receiver1, sender2),
        UnboundedTransport::new(Some(name), receiver2, sender1),
    )
}

#[derive(Debug)]
pub struct UnboundedTransport {
    name: Option<String>,
    receiver: mpsc::UnboundedReceiver<Message>,
    sender: mpsc::UnboundedSender<Message>,
}

impl UnboundedTransport {
    fn new(
        name: Option<String>,
        receiver: mpsc::UnboundedReceiver<Message>,
        sender: mpsc::UnboundedSender<Message>,
    ) -> Self {
        UnboundedTransport {
            name,
            receiver,
            sender,
        }
    }
}

impl Stream for UnboundedTransport {
    type Item = Result<Message, Closed>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Message, Closed>>> {
        Pin::new(&mut self.receiver)
            .poll_next(cx)
            .map(|msg| msg.map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.receiver.size_hint()
    }
}

impl Sink<Message> for UnboundedTransport {
    type Error = Closed;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_ready(cx)
            .map(map_poll_send_error)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Closed> {
        Pin::new(&mut self.sender)
            .start_send(item)
            .map_err(map_send_error)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_flush(cx)
            .map(map_poll_send_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Closed>> {
        Pin::new(&mut self.sender)
            .poll_close(cx)
            .map(map_poll_send_error)
    }
}

impl Transport for UnboundedTransport {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Error type for channel transports.
///
/// Channel transports fail only when either end has been closed or dropped.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Closed;

impl fmt::Display for Closed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("channel closed")
    }
}

impl Error for Closed {}

fn map_send_error(e: SendError) -> Closed {
    if e.is_disconnected() {
        Closed
    } else {
        unreachable!();
    }
}

fn map_poll_send_error<T>(r: Result<T, SendError>) -> Result<T, Closed> {
    r.map_err(map_send_error)
}
