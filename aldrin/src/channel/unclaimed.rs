use super::{Receiver, Sender, UnboundReceiver, UnboundSender};
use crate::{Error, Handle, low_level};
use aldrin_core::ChannelCookie;
use std::fmt;
use std::marker::PhantomData;
use std::task::{Context, Poll};

/// A sender that hasn't been claimed yet by a client.
///
/// When creating and establishing channels, [`UnclaimedSender`s](Self) appear in two places: first,
/// when creating a channel through
/// [`ChannelBuilder::claim_receiver`](`super::ChannelBuilder::claim_receiver`) and second, when
/// [binding an `UnboundSender`](UnboundSender::bind) to a client.
///
/// In the first case, the [`UnclaimedSender`] is typically unbound from its client and then handed
/// over to some other client. The second case appears when receiving an [`UnboundSender`] from
/// e.g. a function call on a service.
pub struct UnclaimedSender<T> {
    inner: low_level::UnclaimedSender,
    phantom: PhantomData<fn(T)>,
}

impl<T> UnclaimedSender<T> {
    pub(crate) fn new(inner: low_level::UnclaimedSender) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the sender into a low-level [`UnclaimedSender`](low_level::UnclaimedSender).
    pub fn into_lowlevel(self) -> low_level::UnclaimedSender {
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
    pub fn cast<U>(self) -> UnclaimedSender<U> {
        UnclaimedSender::new(self.inner)
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
    /// [`PendingReceiver::establish`](super::PendingReceiver::establish) (or the related methods)
    /// will return an error.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle
    ///     .create_channel::<String>()
    ///     .claim_receiver(16)
    ///     .await?;
    ///
    /// // Close the UnclaimedSender:
    /// sender.close().await?;
    ///
    /// // The PendingReceiver will be unable to establish the channel:
    /// let res = receiver.establish().await;
    /// assert_eq!(res.unwrap_err(), Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Unbinds the sender from its client.
    ///
    /// Unbinding is a necessary steps in order send a channel end to another client by
    /// e.g. returning it from a call to a function of a service.
    pub fn unbind(self) -> UnboundSender<T> {
        self.inner.unbind().cast()
    }

    /// Claims the sender and establishes the channel.
    ///
    /// If successful, this fully establishes the channel and unblocks any calls to
    /// [`PendingReceiver::establish`](super::PendingReceiver::establish) (or the related methods).
    ///
    /// ```
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
    pub async fn claim(self) -> Result<Sender<T>, Error> {
        let inner = self.inner.claim().await?;
        Ok(Sender::new(inner))
    }
}

impl<T> fmt::Debug for UnclaimedSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnclaimedSender")
            .field("inner", &self.inner)
            .finish()
    }
}

/// A receiver that hasn't been claimed yet by a client.
///
/// When creating and establishing channels, [`UnclaimedReceiver`s](Self) appear in two places:
/// first, when creating a channel through
/// [`ChannelBuilder::claim_sender`](`super::ChannelBuilder::claim_sender`) and second, when
/// [binding an `UnboundReceiver`](UnboundReceiver::bind) to a client.
///
/// In the first case, the [`UnclaimedReceiver`] is typically unbound from its client and then
/// handed over to some other client. The second case appears when receiving an [`UnboundReceiver`]
/// from e.g. a function call on a service.
pub struct UnclaimedReceiver<T> {
    inner: low_level::UnclaimedReceiver,
    phantom: PhantomData<fn() -> T>,
}

impl<T> UnclaimedReceiver<T> {
    pub(crate) fn new(inner: low_level::UnclaimedReceiver) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the receiver into a low-level [`UnclaimedReceiver`](low_level::UnclaimedReceiver).
    pub fn into_lowlevel(self) -> low_level::UnclaimedReceiver {
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
    pub fn cast<U>(self) -> UnclaimedReceiver<U> {
        UnclaimedReceiver::new(self.inner)
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
    /// [`PendingSender::establish`](super::PendingSender::establish) (or the related methods) will
    /// return an error.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle
    ///     .create_channel::<String>()
    ///     .claim_sender()
    ///     .await?;
    ///
    /// // Close the UnclaimedReceiver:
    /// receiver.close().await?;
    ///
    /// // The PendingSender will be unable to establish the channel:
    /// let res = sender.establish().await;
    /// assert_eq!(res.unwrap_err(), Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Unbinds the receiver from its client.
    ///
    /// Unbinding is a necessary steps in order send a channel end to another client by
    /// e.g. returning it from a call to a function of a service.
    pub fn unbind(self) -> UnboundReceiver<T> {
        self.inner.unbind().cast()
    }

    /// Claims the receiver and establishes the channel.
    ///
    /// If successful, this fully establishes the channel and unblocks any calls to
    /// [`PendingSender::establish`](super::PendingSender::establish) (or the related methods).
    ///
    /// ```
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
    pub async fn claim(self, capacity: u32) -> Result<Receiver<T>, Error> {
        let inner = self.inner.claim(capacity).await?;
        Ok(Receiver::new(inner))
    }
}

impl<T> fmt::Debug for UnclaimedReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnclaimedReceiver")
            .field("inner", &self.inner)
            .finish()
    }
}
