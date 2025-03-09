use super::{RawChannel, Receiver, Sender};
use crate::{Error, Handle};
use aldrin_core::{ChannelCookie, SerializedValue};
use futures_channel::{mpsc, oneshot};
use std::fmt;
use std::future::{self, Future};
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A sender that is waiting for the channel to be established.
///
/// When creating channels, one of the channel ends is always claimed by the local client, depending
/// on whether [`ChannelBuilder::claim_sender`](super::ChannelBuilder::claim_sender) or
/// [`claim_receiver`](super::ChannelBuilder::claim_receiver) is used. If the sender is claimed,
/// then this will return a [`PendingSender`].
///
/// [`PendingSender`s](Self) are an intermediate step before the channel is fully established. It's
/// purpose is to wait until the other channel end is claimed as well.
#[derive(Debug)]
pub struct PendingSender {
    inner: RawChannel<true>,
    recv: OneshotReceiver<Result<(mpsc::UnboundedReceiver<u32>, u32), Error>>,
}

impl PendingSender {
    pub(crate) fn new(
        client: Handle,
        cookie: ChannelCookie,
        recv: oneshot::Receiver<Result<(mpsc::UnboundedReceiver<u32>, u32), Error>>,
    ) -> Self {
        Self {
            inner: RawChannel::claimed(client, cookie),
            recv: OneshotReceiver::new(recv),
        }
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(&self) -> &Handle {
        self.inner.client()
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts to a high-level [`PendingSender`](crate::PendingSender) by binding an item type
    /// `T`.
    pub fn cast<T>(self) -> crate::PendingSender<T> {
        crate::PendingSender::new(self)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.recv.close();
        self.inner.poll_close(cx)
    }

    /// Closes the sender.
    ///
    /// This will prevent the channel from being established. Any calls to
    /// [`UnclaimedReceiver::claim`](super::UnclaimedReceiver::claim) will return an error.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle
    ///     .create_low_level_channel()
    ///     .claim_sender()
    ///     .await?;
    ///
    /// // Close the PendingSender:
    /// sender.close().await?;
    ///
    /// // Trying to claim the UnclaimedReceiver will fail:
    /// let res = receiver.claim(16).await;
    /// assert_eq!(res.unwrap_err(), Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    /// Polls the channel for whether it has been established.
    ///
    /// When this method returns `Poll::Ready(())`, then [`establish`](Self::establish) will resolve
    /// immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub fn poll_wait_established(&mut self, cx: &mut Context) -> Poll<()> {
        self.recv.poll(cx)
    }

    /// Waits until the channel has been established.
    ///
    /// When this method returns, then [`establish`](Self::establish) will resolve immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub async fn wait_established(&mut self) {
        self.recv.wait().await
    }

    /// Waits until the channel has been established and returns a [`Sender`].
    ///
    /// It can occasionally be useful to only wait until the channel is established, but without
    /// converting `self` to a [`Sender`]. This can e.g. happen in `select!` macros or similar
    /// situations. The reason is, that this method takes `self` by value and is not
    /// cancel-safe. Use [`wait_established`](Self::wait_established) in such cases.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle
    ///     .create_low_level_channel()
    ///     .claim_sender()
    ///     .await?;
    ///
    /// // Claim the receiver:
    /// let mut receiver = receiver.claim(16).await?;
    ///
    /// // The channel is now established:
    /// let mut sender = sender.establish().await?;
    ///
    /// sender.send_item("Hello :)").await?;
    /// let item = receiver.next_item::<String>().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(self) -> Result<Sender, Error> {
        let (capacity_added, capacity) = self
            .recv
            .wait_and_take()
            .await
            .map_err(|_| Error::Shutdown)??;

        Ok(Sender::new(self.inner, capacity_added, capacity))
    }
}

/// A receiver that is waiting for the channel to be established.
///
/// When creating channels, one of the channel ends is always claimed by the local client, depending
/// on whether [`ChannelBuilder::claim_sender`](super::ChannelBuilder::claim_sender) or
/// [`claim_receiver`](super::ChannelBuilder::claim_receiver) is used. If the receiver is claimed,
/// then this will return a [`PendingReceiver`].
///
/// [`PendingReceiver`s](Self) are an intermediate step before the channel is fully
/// established. It's purpose is to wait until the other channel end is claimed as well.
#[derive(Debug)]
pub struct PendingReceiver {
    inner: RawChannel<false>,
    recv: OneshotReceiver<Result<mpsc::UnboundedReceiver<SerializedValue>, Error>>,
    capacity: NonZeroU32,
}

impl PendingReceiver {
    pub(crate) fn new(
        client: Handle,
        cookie: ChannelCookie,
        recv: oneshot::Receiver<Result<mpsc::UnboundedReceiver<SerializedValue>, Error>>,
        capacity: NonZeroU32,
    ) -> Self {
        Self {
            inner: RawChannel::claimed(client, cookie),
            recv: OneshotReceiver::new(recv),
            capacity,
        }
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(&self) -> &Handle {
        self.inner.client()
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts to a high-level [`PendingSender`](crate::PendingSender) by binding an item type
    /// `T`.
    pub fn cast<T>(self) -> crate::PendingReceiver<T> {
        crate::PendingReceiver::new(self)
    }

    /// Initiates closing the receiver and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.recv.close();
        self.inner.poll_close(cx)
    }

    /// Closes the receiver.
    ///
    /// This will prevent the channel from being established. Any calls to
    /// [`UnclaimedSender::claim`](super::UnclaimedSender::claim) will return an error.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle
    ///     .create_low_level_channel()
    ///     .claim_receiver(16)
    ///     .await?;
    ///
    /// // Close the PendingReceiver:
    /// receiver.close().await?;
    ///
    /// // Trying to claim the UnclaimedSender will fail:
    /// let res = sender.claim().await;
    /// assert_eq!(res.unwrap_err(), Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    /// Polls the channel for whether it has been established.
    ///
    /// When this method returns `Poll::Ready(())`, then [`establish`](Self::establish) will resolve
    /// immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub fn poll_wait_established(&mut self, cx: &mut Context) -> Poll<()> {
        self.recv.poll(cx)
    }

    /// Waits until the channel has been established.
    ///
    /// When this method returns, then [`establish`](Self::establish) will resolve immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub async fn wait_established(&mut self) {
        self.recv.wait().await
    }

    /// Waits until the channel has been established and returns a [`Receiver`].
    ///
    /// It can occasionally be useful to only wait until the channel is established, but without
    /// converting `self` to a [`Receiver`]. This can e.g. happen in `select!` macros or similar
    /// situations. The reason is, that this method takes `self` by value and is not
    /// cancel-safe. Use [`wait_established`](Self::wait_established) in such cases.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle
    ///     .create_low_level_channel()
    ///     .claim_receiver(16)
    ///     .await?;
    ///
    /// // Claim the sender:
    /// let mut sender = sender.claim().await?;
    ///
    /// // The channel is now established:
    /// let mut receiver = receiver.establish().await?;
    ///
    /// sender.send_item("Hello :)").await?;
    /// let item = receiver.next_item::<String>().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(self) -> Result<Receiver, Error> {
        let items = self
            .recv
            .wait_and_take()
            .await
            .map_err(|_| Error::Shutdown)??;

        Ok(Receiver::new(self.inner, items, self.capacity))
    }
}

enum OneshotReceiver<T> {
    Pending(oneshot::Receiver<T>),
    Ready(Result<T, oneshot::Canceled>),
}

impl<T> OneshotReceiver<T> {
    fn new(inner: oneshot::Receiver<T>) -> Self {
        Self::Pending(inner)
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        match self {
            Self::Pending(inner) => match Pin::new(inner).poll(cx) {
                Poll::Ready(res) => {
                    *self = Self::Ready(res);
                    Poll::Ready(())
                }

                Poll::Pending => Poll::Pending,
            },

            Self::Ready(_) => Poll::Ready(()),
        }
    }

    async fn wait(&mut self) {
        future::poll_fn(|cx| self.poll(cx)).await
    }

    async fn wait_and_take(self) -> Result<T, oneshot::Canceled> {
        match self {
            Self::Pending(inner) => inner.await,
            Self::Ready(res) => res,
        }
    }

    fn close(&mut self) {
        *self = Self::Ready(Err(oneshot::Canceled));
    }
}

impl<T> fmt::Debug for OneshotReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pending(inner) => f
                .debug_tuple("OneshotReceiver::Pending")
                .field(inner)
                .finish(),

            Self::Ready(Ok(_)) => write!(f, "OneshotReceiver::Ready(Ok(..))"),

            Self::Ready(Err(e)) => f
                .debug_tuple("OneshotReceiver::Ready")
                .field(&Err::<(), _>(e))
                .finish(),
        }
    }
}
