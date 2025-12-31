use super::{Receiver, Sender};
use crate::{Error, Handle, low_level};
use aldrin_core::ChannelCookie;
use std::fmt;
use std::marker::PhantomData;
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
pub struct PendingSender<T> {
    inner: low_level::PendingSender,
    phantom: PhantomData<fn(T)>,
}

impl<T> PendingSender<T> {
    pub(crate) fn new(inner: low_level::PendingSender) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the sender into a low-level [`PendingSender`](low_level::PendingSender).
    pub fn into_lowlevel(self) -> low_level::PendingSender {
        self.inner
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(&self) -> &Handle {
        self.inner.client()
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts the item type to a different type `U`.
    pub fn cast<U>(self) -> PendingSender<U> {
        PendingSender::new(self.inner)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
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
    ///     .create_channel::<String>()
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
        self.inner.close().await
    }

    /// Polls the channel for whether it has been established.
    ///
    /// When this method returns `Poll::Ready(())`, then [`establish`](Self::establish) will resolve
    /// immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub fn poll_wait_established(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_wait_established(cx)
    }

    /// Waits until the channel has been established.
    ///
    /// When this method returns, then [`establish`](Self::establish) will resolve immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub async fn wait_established(&mut self) {
        self.inner.wait_established().await
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
    ///     .create_channel::<String>()
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
    /// let item = receiver.next_item().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(self) -> Result<Sender<T>, Error> {
        let inner = self.inner.establish().await?;
        Ok(Sender::new(inner))
    }
}

impl<T> fmt::Debug for PendingSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingSender")
            .field("inner", &self.inner)
            .finish()
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
pub struct PendingReceiver<T> {
    inner: low_level::PendingReceiver,
    phantom: PhantomData<fn() -> T>,
}

impl<T> PendingReceiver<T> {
    pub(crate) fn new(inner: low_level::PendingReceiver) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the receiver into a low-level [`PendingReceiver`](low_level::PendingReceiver).
    pub fn into_lowlevel(self) -> low_level::PendingReceiver {
        self.inner
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(&self) -> &Handle {
        self.inner.client()
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts the item type to a different type `U`.
    pub fn cast<U>(self) -> PendingReceiver<U> {
        PendingReceiver::new(self.inner)
    }

    /// Initiates closing the receiver and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
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
    ///     .create_channel::<String>()
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
        self.inner.close().await
    }

    /// Polls the channel for whether it has been established.
    ///
    /// When this method returns `Poll::Ready(())`, then [`establish`](Self::establish) will resolve
    /// immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub fn poll_wait_established(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_wait_established(cx)
    }

    /// Waits until the channel has been established.
    ///
    /// When this method returns, then [`establish`](Self::establish) will resolve immediately.
    ///
    /// Note that this method does not indicate errors. You must use [`establish`](Self::establish)
    /// to check whether the channel was successfully established.
    pub async fn wait_established(&mut self) {
        self.inner.wait_established().await
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
    ///     .create_channel::<String>()
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
    /// let item = receiver.next_item().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(self) -> Result<Receiver<T>, Error> {
        let inner = self.inner.establish().await?;
        Ok(Receiver::new(inner))
    }
}

impl<T> fmt::Debug for PendingReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingReceiver")
            .field("inner", &self.inner)
            .finish()
    }
}
