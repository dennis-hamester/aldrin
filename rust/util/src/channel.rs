//! Transports for connecting brokers and clients in the same process.

use aldrin_proto::{AsyncTransport, Message};
use futures_channel::mpsc;
use futures_core::stream::Stream;
use std::error::Error;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Creates a pair of bounded channel transports.
///
/// Both transports have a separate fifo for receiving [`Message`s](Message). If either fifo is
/// full, this will cause backpressure to the sender.
pub fn channel(fifo_size: usize) -> (Channel, Channel) {
    let (sender1, receiver1) = mpsc::channel(fifo_size);
    let (sender2, receiver2) = mpsc::channel(fifo_size);

    (
        Channel::new(None, receiver1, sender2),
        Channel::new(None, receiver2, sender1),
    )
}

/// Creates a pair of bounded channel transports with a name.
///
/// Both transports have a separate fifo for receiving [`Message`s](Message). If either fifo runs
/// full, backpressure will be applied to the sender.
pub fn channel_with_name<N>(fifo_size: usize, name: N) -> (Channel, Channel)
where
    N: Into<String>,
{
    let name = name.into();
    let (sender1, receiver1) = mpsc::channel(fifo_size);
    let (sender2, receiver2) = mpsc::channel(fifo_size);

    (
        Channel::new(Some(name.clone()), receiver1, sender2),
        Channel::new(Some(name), receiver2, sender1),
    )
}

/// A bounded channels-based transport for connecting a broker and a client in the same process.
///
/// Bounded transports have an internal fifo for receiving [`Message`s](Message). If this runs full,
/// backpressure will be applied to the sender.
#[derive(Debug)]
pub struct Channel {
    name: Option<String>,
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
}

impl Channel {
    fn new(
        name: Option<String>,
        receiver: mpsc::Receiver<Message>,
        sender: mpsc::Sender<Message>,
    ) -> Self {
        Channel {
            name,
            receiver,
            sender,
        }
    }
}

/// Error type when using channels as a transport.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Disconnected;

impl fmt::Display for Disconnected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("disconnected")
    }
}

impl Error for Disconnected {}

impl AsyncTransport for Channel {
    type Error = Disconnected;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Disconnected>> {
        Pin::new(&mut self.receiver).poll_next(cx).map(Ok)
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<bool, Disconnected>> {
        match self.sender.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(true)),
            Poll::Ready(Err(_)) => Poll::Ready(Ok(false)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Disconnected> {
        self.sender.start_send(msg).map_err(|_| Disconnected)
    }

    fn send_poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<(), Disconnected>> {
        self.sender.close_channel();
        Poll::Ready(Ok(()))
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Creates a pair of unbounded channel transports.
pub fn unbounded() -> (Unbounded, Unbounded) {
    let (sender1, receiver1) = mpsc::unbounded();
    let (sender2, receiver2) = mpsc::unbounded();

    (
        Unbounded::new(None, receiver1, sender2),
        Unbounded::new(None, receiver2, sender1),
    )
}

/// Creates a pair of unbounded channel transports with a name.
pub fn unbounded_with_name<N>(name: N) -> (Unbounded, Unbounded)
where
    N: Into<String>,
{
    let name = name.into();
    let (sender1, receiver1) = mpsc::unbounded();
    let (sender2, receiver2) = mpsc::unbounded();

    (
        Unbounded::new(Some(name.clone()), receiver1, sender2),
        Unbounded::new(Some(name), receiver2, sender1),
    )
}

/// An unbounded channels-based transport for connecting a broker and a client in the same process.
#[derive(Debug)]
pub struct Unbounded {
    name: Option<String>,
    receiver: mpsc::UnboundedReceiver<Message>,
    sender: mpsc::UnboundedSender<Message>,
}

impl Unbounded {
    fn new(
        name: Option<String>,
        receiver: mpsc::UnboundedReceiver<Message>,
        sender: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Unbounded {
            name,
            receiver,
            sender,
        }
    }
}

impl AsyncTransport for Unbounded {
    type Error = Disconnected;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Disconnected>> {
        Pin::new(&mut self.receiver).poll_next(cx).map(Ok)
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<bool, Disconnected>> {
        match self.sender.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(true)),
            Poll::Ready(Err(_)) => Poll::Ready(Ok(false)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Disconnected> {
        self.sender.start_send(msg).map_err(|_| Disconnected)
    }

    fn send_poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        self.sender.close_channel();
        Poll::Ready(Ok(()))
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
