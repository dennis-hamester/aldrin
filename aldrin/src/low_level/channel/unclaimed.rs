use super::{RawChannel, Receiver, Sender, UnboundReceiver, UnboundSender};
use crate::channel as high_level;
use crate::core::ChannelCookie;
use crate::error::Error;
use crate::handle::Handle;
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
#[derive(Debug)]
pub struct UnclaimedSender {
    inner: RawChannel<true>,
}

impl UnclaimedSender {
    pub(crate) fn new(client: Handle, cookie: ChannelCookie) -> Self {
        Self {
            inner: RawChannel::unclaimed(client, cookie),
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

    /// Casts to a high-level [`UnclaimedSender`](high_level::UnclaimedSender) by binding an item
    /// type `T`.
    pub fn cast<T: ?Sized>(self) -> high_level::UnclaimedSender<T> {
        high_level::UnclaimedSender::new(self)
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
    ///     .create_low_level_channel()
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
    pub fn unbind(self) -> UnboundSender {
        let cookie = self.inner.unbind();
        UnboundSender::new(cookie)
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
    pub async fn claim(mut self) -> Result<Sender, Error> {
        self.inner.set_claimed();
        let (capacity_added, capacity) = self.client().claim_sender(self.cookie()).await?;
        Ok(Sender::new(self.inner, capacity_added, capacity))
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
#[derive(Debug)]
pub struct UnclaimedReceiver {
    inner: RawChannel<false>,
}

impl UnclaimedReceiver {
    pub(crate) fn new(client: Handle, cookie: ChannelCookie) -> Self {
        Self {
            inner: RawChannel::unclaimed(client, cookie),
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

    /// Casts to a high-level [`UnclaimedReceiver`](high_level::UnclaimedReceiver) by binding an
    /// item type `T`.
    pub fn cast<T>(self) -> high_level::UnclaimedReceiver<T> {
        high_level::UnclaimedReceiver::new(self)
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
    ///     .create_low_level_channel()
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
    pub fn unbind(self) -> UnboundReceiver {
        let cookie = self.inner.unbind();
        UnboundReceiver::new(cookie)
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
    pub async fn claim(mut self, capacity: u32) -> Result<Receiver, Error> {
        self.inner.set_claimed();

        let (items, max_capacity) = self
            .client()
            .claim_receiver(self.cookie(), capacity)
            .await?;

        Ok(Receiver::new(self.inner, items, max_capacity))
    }
}
