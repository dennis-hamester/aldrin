use super::Message;
use pin_project::pin_project;
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
/// Transports shut down only implicitly when dropped or in the case of errors. There is no
/// explicit shutdown method, because the Aldrin protocol defines the [`Shutdown`] message, after
/// which users of this trait are expected to drop the transport. Any unexpected shutdown must be
/// signaled with an [`Error`] by the implementation.
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
/// [`Shutdown`]: Message::Shutdown
pub trait AsyncTransport {
    /// Error type when sending or receiving messages.
    type Error;

    /// Attempts to receive the next message.
    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>>;

    /// Prepares the transport for sending a message.
    ///
    /// This method must be called before sending a [`Message`] with
    /// [`send_start`](AsyncTransport::send_start). Only when it returns `Poll::Ready(Ok(()))` is
    /// the transport ready to start sending a single [`Message`].
    ///
    /// The transport may be implicitly flushed, fully or partially, when this method is called.
    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Begins sending a message.
    ///
    /// Every call to this method must be preceded by a successful call to
    /// [`send_poll_ready`](AsyncTransport::send_poll_ready).
    ///
    /// Sending a [`Message`] may flush the transport, but does not necessarily do so. Thus, even
    /// when `Ok(())` is returned, the [`Message`] may not yet be delivered to the remote end of
    /// the transport. Use [`send_poll_flush`](AsyncTransport::send_poll_flush) to explicitly flush
    /// the transport.
    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error>;

    /// Attempts to flush the transport.
    ///
    /// Flushing must deliver _all_ prior [`Message`s](Message) to the remote end of the transport.
    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Returns an optional transport name.
    ///
    /// The name does not need to be unique for a specific broker. The default implementation
    /// returns `None`.
    ///
    /// This method may be called at any time, regardless of the error state.
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

    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>> {
        self.get_mut().as_mut().receive_poll(cx)
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().send_poll_ready(cx)
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        self.get_mut().as_mut().send_start(msg)
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().send_poll_flush(cx)
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
    ) -> Poll<Result<Message, Self::Error>> {
        Pin::new(&mut **self).receive_poll(cx)
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
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
    ) -> Poll<Result<Message, Self::Error>> {
        T::receive_poll(Pin::new(&mut **self), cx)
    }

    fn send_poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
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

    fn receive_poll_unpin(&mut self, cx: &mut Context) -> Poll<Result<Message, Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).receive_poll(cx)
    }

    fn send_poll_ready_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
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

    fn map_err<F, E>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapError {
            transport: self,
            map_err: f,
        }
    }
}

impl<T> AsyncTransportExt for T where T: AsyncTransport {}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receive<'a, T>(&'a mut T)
where
    T: AsyncTransport + Unpin + ?Sized;

impl<'a, T> Future for Receive<'a, T>
where
    T: AsyncTransport + Unpin + ?Sized,
{
    type Output = Result<Message, T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.0.receive_poll_unpin(cx)
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
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
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.t.send_poll_ready_unpin(cx) {
            Poll::Ready(Ok(())) => {
                let msg = self.msg.take().unwrap();
                if let Err(e) = self.t.send_start_unpin(msg) {
                    return Poll::Ready(Err(e));
                }
                Poll::Ready(Ok(()))
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
#[must_use = "futures do nothing unless you `.await` or poll them"]
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
#[must_use = "futures do nothing unless you `.await` or poll them"]
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
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let SendFlushInner::Send(ref mut send) = self.0 {
            match Pin::new(send).poll(cx) {
                Poll::Ready(Ok(())) => {}
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
            SendFlushInner::Flush(ref mut flush) => Pin::new(flush).poll(cx),
            _ => unreachable!(),
        }
    }
}

#[pin_project]
#[derive(Debug)]
pub struct MapError<T, F> {
    #[pin]
    transport: T,
    map_err: F,
}

impl<T, F, E> AsyncTransport for MapError<T, F>
where
    T: AsyncTransport,
    F: FnMut(T::Error) -> E,
{
    type Error = E;

    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>> {
        let mut this = self.project();
        this.transport.receive_poll(cx).map_err(&mut this.map_err)
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        this.transport
            .send_poll_ready(cx)
            .map_err(&mut this.map_err)
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        let mut this = self.project();
        this.transport.send_start(msg).map_err(&mut this.map_err)
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        this.transport
            .send_poll_flush(cx)
            .map_err(&mut this.map_err)
    }

    fn name(&self) -> Option<&str> {
        self.transport.name()
    }
}
