use super::Message;
use std::future::Future;
use std::mem;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Bidirectional asynchronous message transport.
///
/// This trait represents the core abstraction used throughout Aldrin for communication between
/// clients and the broker. It is essentially a combination of `Futures`' [`Stream`] trait for
/// receiving [`Message`s] and the [`Sink`] trait for sending.
///
/// Implementations must be reliable and ordered. Reliable means that [`Message`s] must not get
/// corrupted. Ordered means that [`Message`s] must be delivered in the same order they were
/// sent.
///
/// Typical implementations include:
///
/// - TCP/IP across a network.
/// - Unix domain sockets (`SOCK_STREAM`) between processes on a single machine.
/// - [Channels] inside a single process.
///
/// # Shutdown
///
/// The transport can be shut down locally or by the remote end. It is expected that a shutdown by
/// either side can be observed by the other.
///
/// Local shutdown is initiated by [`poll_shutdown`].
///
/// A remote shutdown can be observed by one of the following:
///
/// - [`receive_poll`] returns `Poll::Ready(Ok(None))` (this may also happen after shutting down
///   locally).
/// - [`send_poll_ready`] returns `Poll::Ready(Ok(false))`.
///
/// # Errors
///
/// All methods may return an [`Error`] at any time. Afterwards, the transport should be considered
/// unusable. Implementations may panic in any further method calls.
///
/// [Channels]: https://docs.rs/futures/latest/futures/channel/mpsc/index.html
/// [`Error`]: AsyncTransport::Error
/// [`Message`s]: Message
/// [`Sink`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`poll_shutdown`]: AsyncTransport::poll_shutdown
/// [`receive_poll`]: AsyncTransport::receive_poll
/// [`send_poll_ready`]: AsyncTransport::send_poll_ready
pub trait AsyncTransport {
    /// Error type when sending or receiving messages.
    type Error;

    /// Attempts to receive the next message.
    ///
    /// If this method returns `Poll::Ready(Ok(None))`, then the last message has been
    /// received and the transport has shut down.
    ///
    /// # Panics
    ///
    /// This method may panic in the following situations:
    ///
    /// - After it has returned `Poll::Ready(Ok(None))`.
    /// - After this or any other method indicated an [`Error`](AsyncTransport::Error).
    fn receive_poll(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Self::Error>>;

    /// Prepares the transport for sending a message.
    ///
    /// This method must be called before sending a [`Message`] with
    /// [`send_start`](AsyncTransport::send_start). Only when it returns `Poll::Ready(Ok(true))` is
    /// the transport ready to start sending a single [`Message`].
    ///
    /// If the remote end of the transport has shut down, `Poll::Ready(Ok(false))` will be returned.
    ///
    /// The transport may be implicitly flushed, fully or partially, when this method is called.
    ///
    /// # Panics
    ///
    /// This method may panic in the following situations:
    ///
    /// - After explicit shutdown with [`poll_shutdown`](AsyncTransport::poll_shutdown).
    /// - After returning `Poll::Ready(Ok(false))`.
    /// - After [`receive_poll`](AsyncTransport::receive_poll) returned `Poll::Ready(Ok(None))`.
    /// - After this or any other method indicated an [`Error`](AsyncTransport::Error).
    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<bool, Self::Error>>;

    /// Begins sending a message.
    ///
    /// Every call to this method must be preceded by a successful call to
    /// [`send_poll_ready`](AsyncTransport::send_poll_ready).
    ///
    /// When sending has been successfully initiated, `Ok(())` is returned. It is an error, if the
    /// transport becomes unable to send the whole [`Message`] (due e.g. a shutdown).
    ///
    /// Sending a [`Message`] may flush the transport, but does not necessarily do so. Thus, even
    /// when `Ok(())` is returned, the [`Message`] may not yet be delivered to the remote end of
    /// the transport. Use [`send_poll_flush`](AsyncTransport::send_poll_flush) to explicitly flush
    /// the transport.
    ///
    /// # Panics
    ///
    /// This method may panic in the following situations:
    ///
    /// - If called without preparing the transport beforehand with
    ///   [`send_poll_ready`](AsyncTransport::send_poll_ready).
    /// - After explicit shutdown with [`poll_shutdown`](AsyncTransport::poll_shutdown).
    /// - After [`receive_poll`](AsyncTransport::receive_poll) returned `Poll::Ready(Ok(None))`.
    /// - After this or any other method indicated an [`Error`](AsyncTransport::Error).
    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error>;

    /// Attempts to flush the transport.
    ///
    /// Flushing will cause _all_ prior [`Message`s](Message) to be delivered to the remote end of
    /// the transport.
    ///
    /// When flushing has been successful, `Poll::Ready(Ok())` is returned. It is an error if the
    /// transport shuts down before it has been fully flushed.
    ///
    /// # Panics
    ///
    /// This method may panic in the following situations:
    ///
    /// - After explicit shutdown with [`poll_shutdown`](AsyncTransport::poll_shutdown).
    /// - After [`receive_poll`](AsyncTransport::receive_poll) returned `Poll::Ready(Ok(None))`.
    /// - After this or any other method indicated an [`Error`](AsyncTransport::Error).
    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Attempts to shut down the transport.
    ///
    /// `Poll::Ready(Ok())` indicates that the transport has been successfully shut
    /// down. Afterwards, the transport may still be drained of received [`Message`s](Message)
    /// until [`receive_poll`](AsyncTransport::receive_poll) returns
    /// `Poll::Ready(Ok(None))`. However, no more [`Message`s](Message) may be sent.
    ///
    /// # Panics
    ///
    /// This method may panic in the following situations:
    ///
    /// - After a successful shutdown.
    /// - After [`receive_poll`](AsyncTransport::receive_poll) returned `Poll::Ready(Ok(None))`.
    /// - After this or any other method indicated an [`Error`](AsyncTransport::Error).
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Returns an optional transport name.
    ///
    /// The name does not need to be unique for a specific broker. The default implementation return
    /// `None`.
    ///
    /// This method may be called at any time, regardless of the shutdown or error state.
    fn name(&self) -> Option<&str> {
        None
    }
}

impl<T> AsyncTransport for Pin<T>
where
    T: DerefMut + Unpin,
    T::Target: AsyncTransport,
{
    type Error = <T::Target as AsyncTransport>::Error;

    fn receive_poll(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Self::Error>> {
        self.get_mut().as_mut().receive_poll(cx)
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<bool, Self::Error>> {
        self.get_mut().as_mut().send_poll_ready(cx)
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        self.get_mut().as_mut().send_start(msg)
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().send_poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().poll_shutdown(cx)
    }

    fn name(&self) -> Option<&str> {
        (**self).name()
    }
}

impl<T> AsyncTransport for Box<T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Error = T::Error;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Self::Error>> {
        Pin::new(&mut **self).receive_poll(cx)
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<bool, Self::Error>> {
        Pin::new(&mut **self).send_poll_ready(cx)
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        Pin::new(&mut **self).send_start(msg)
    }

    fn send_poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut **self).send_poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut **self).poll_shutdown(cx)
    }

    fn name(&self) -> Option<&str> {
        (**self).name()
    }
}

impl<T> AsyncTransport for &mut T
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Error = T::Error;

    fn receive_poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Self::Error>> {
        T::receive_poll(Pin::new(&mut **self), cx)
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<bool, Self::Error>> {
        T::send_poll_ready(Pin::new(&mut **self), cx)
    }

    fn send_start(mut self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        T::send_start(Pin::new(&mut **self), msg)
    }

    fn send_poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
        T::send_poll_flush(Pin::new(&mut **self), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        T::poll_shutdown(Pin::new(&mut **self), cx)
    }

    fn name(&self) -> Option<&str> {
        T::name(*self)
    }
}

pub trait AsyncTransportExt: AsyncTransport {
    fn receive(&mut self) -> Receive<'_, Self>
    where
        Self: Unpin,
    {
        Receive(self)
    }

    fn send(&mut self, msg: Message) -> Send<'_, Self>
    where
        Self: Unpin,
    {
        Send {
            t: self,
            msg: Some(msg),
        }
    }

    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Unpin,
    {
        Flush(self)
    }

    fn send_and_flush(&mut self, msg: Message) -> SendFlush<'_, Self>
    where
        Self: Unpin,
    {
        SendFlush(SendFlushInner::Send(self.send(msg)))
    }

    fn shutdown(&mut self) -> Shutdown<'_, Self>
    where
        Self: Unpin,
    {
        Shutdown(self)
    }

    fn receive_poll_unpin(&mut self, cx: &mut Context) -> Poll<Result<Option<Message>, Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).receive_poll(cx)
    }

    fn send_poll_ready_unpin(&mut self, cx: &mut Context) -> Poll<Result<bool, Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).send_poll_ready(cx)
    }

    fn send_start_unpin(&mut self, msg: Message) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        Pin::new(self).send_start(msg)
    }

    fn send_poll_flush_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).send_poll_flush(cx)
    }

    fn poll_shutdown_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_shutdown(cx)
    }
}

impl<T> AsyncTransportExt for T where T: AsyncTransport {}

#[derive(Debug)]
pub struct Receive<'a, T>(&'a mut T)
where
    T: AsyncTransport + Unpin + ?Sized;

impl<'a, T> Future for Receive<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<Option<Message>, T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.0.receive_poll_unpin(cx)
    }
}

#[derive(Debug)]
pub struct Send<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    t: &'a mut T,
    msg: Option<Message>,
}

impl<'a, T> Future for Send<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<bool, T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.t.send_poll_ready_unpin(cx) {
            Poll::Ready(Ok(true)) => {
                let msg = self.msg.take().unwrap();
                if let Err(e) = self.t.send_start_unpin(msg) {
                    return Poll::Ready(Err(e));
                }
                Poll::Ready(Ok(true))
            }

            Poll::Ready(Ok(false)) => {
                self.msg.take();
                Poll::Ready(Ok(false))
            }

            Poll::Ready(Err(e)) => {
                self.msg.take();
                Poll::Ready(Err(e))
            }

            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct Flush<'a, T>(&'a mut T)
where
    T: AsyncTransport + Unpin + ?Sized;

impl<'a, T> Future for Flush<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.0.send_poll_flush_unpin(cx)
    }
}

#[derive(Debug)]
pub struct SendFlush<'a, T>(SendFlushInner<'a, T>)
where
    T: AsyncTransport + Unpin + ?Sized;

#[derive(Debug)]
enum SendFlushInner<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    Send(Send<'a, T>),
    Flush(Flush<'a, T>),
    None,
}

impl<'a, T> Future for SendFlush<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<bool, T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let SendFlushInner::Send(ref mut send) = self.0 {
            match Pin::new(send).poll(cx) {
                Poll::Ready(Ok(true)) => {}
                p => return p,
            }

            let mut tmp = SendFlushInner::None;
            mem::swap(&mut tmp, &mut self.0);
            let t = match tmp {
                SendFlushInner::Send(s) => s.t,
                _ => unreachable!(),
            };
            self.0 = SendFlushInner::Flush(Flush(t));
        }

        match self.0 {
            SendFlushInner::Flush(ref mut flush) => Pin::new(flush).poll(cx).map_ok(|_| true),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Shutdown<'a, T>(&'a mut T)
where
    T: AsyncTransport + Unpin + ?Sized;

impl<'a, T> Future for Shutdown<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.0.poll_shutdown_unpin(cx)
    }
}
