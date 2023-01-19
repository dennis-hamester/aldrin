//! Channels-based Aldrin transports.
//!
//! Channels-based transports can be used to efficiently connect client and broker within the same
//! process.
//!
//! The transports come in two flavors, [`Bounded`] and [`Unbounded`]. [`Bounded`] will cause
//! back-pressure to the sender when an internal fifo runs full, whereas [`Unbounded`] never blocks
//! (asynchronously).
//!
//! # Examples
//!
//! ```
//! use aldrin_broker::Broker;
//! use aldrin_client::Client;
//! use futures::future;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a broker:
//!     let broker = Broker::new();
//!     let mut broker_handle = broker.handle().clone();
//!     let broker_join = tokio::spawn(broker.run());
//!
//!     // Connect a client with the Bounded transport:
//!     let (t1, t2) = aldrin_channel::bounded(16);
//!     let (connection1, client1) =
//!         future::join(broker_handle.connect(t1), Client::connect(t2)).await;
//!     let connection1 = connection1?;
//!     let client1 = client1?;
//!     tokio::spawn(connection1.run());
//!     let client1_handle = client1.handle().clone();
//!     let client1_join = tokio::spawn(client1.run());
//!
//!     // Connect a client with the Unbounded transport:
//!     let (t1, t2) = aldrin_channel::unbounded();
//!     let (connection2, client2) =
//!         future::join(broker_handle.connect(t1), Client::connect(t2)).await;
//!     let connection2 = connection2?;
//!     let client2 = client2?;
//!     tokio::spawn(connection2.run());
//!     let client2_handle = client2.handle().clone();
//!     let client2_join = tokio::spawn(client2.run());
//!
//!     // Shut everything down again:
//!     broker_handle.shutdown_idle().await;
//!     client1_handle.shutdown();
//!     client1_join.await??;
//!     client2_handle.shutdown();
//!     client2_join.await??;
//!     broker_join.await?;
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use aldrin_proto::message::Message;
use aldrin_proto::transport::AsyncTransport;
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
pub fn bounded(fifo_size: usize) -> (Bounded, Bounded) {
    let (sender1, receiver1) = mpsc::channel(fifo_size);
    let (sender2, receiver2) = mpsc::channel(fifo_size);

    (
        Bounded::new(receiver1, sender2),
        Bounded::new(receiver2, sender1),
    )
}

/// A bounded channels-based transport for connecting a broker and a client in the same process.
///
/// Bounded transports have an internal fifo for receiving [`Message`s](Message). If this runs full,
/// backpressure will be applied to the sender.
#[derive(Debug)]
pub struct Bounded {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
}

impl Bounded {
    fn new(receiver: mpsc::Receiver<Message>, sender: mpsc::Sender<Message>) -> Self {
        Bounded { receiver, sender }
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

impl AsyncTransport for Bounded {
    type Error = Disconnected;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Message, Disconnected>> {
        match Pin::new(&mut self.receiver).poll_next(cx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Ok(msg)),
            Poll::Ready(None) => Poll::Ready(Err(Disconnected)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Disconnected>> {
        match self.sender.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(Disconnected)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Disconnected> {
        self.sender.start_send(msg).map_err(|_| Disconnected)
    }

    fn send_poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        Poll::Ready(Ok(()))
    }
}

/// Creates a pair of unbounded channel transports.
pub fn unbounded() -> (Unbounded, Unbounded) {
    let (sender1, receiver1) = mpsc::unbounded();
    let (sender2, receiver2) = mpsc::unbounded();

    (
        Unbounded::new(receiver1, sender2),
        Unbounded::new(receiver2, sender1),
    )
}

/// An unbounded channels-based transport for connecting a broker and a client in the same process.
#[derive(Debug)]
pub struct Unbounded {
    receiver: mpsc::UnboundedReceiver<Message>,
    sender: mpsc::UnboundedSender<Message>,
}

impl Unbounded {
    fn new(
        receiver: mpsc::UnboundedReceiver<Message>,
        sender: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Unbounded { receiver, sender }
    }
}

impl AsyncTransport for Unbounded {
    type Error = Disconnected;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Message, Disconnected>> {
        match Pin::new(&mut self.receiver).poll_next(cx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Ok(msg)),
            Poll::Ready(None) => Poll::Ready(Err(Disconnected)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        match self.sender.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(Disconnected)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Disconnected> {
        self.sender.start_send(msg).map_err(|_| Disconnected)
    }

    fn send_poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Disconnected>> {
        Poll::Ready(Ok(()))
    }
}
