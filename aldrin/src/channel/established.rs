use crate::{Error, Handle, low_level};
use aldrin_core::tags::PrimaryTag;
use aldrin_core::{ChannelCookie, Deserialize, DeserializePrimary, Serialize, SerializePrimary};
use futures_core::stream::{FusedStream, Stream};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, future};

/// The sending end of an established channel.
///
/// [`Sender`s](Self) are acquired by either
/// [`PendingSender::establish`](super::PendingSender::establish) or
/// [`UnclaimedSender::claim`](super::UnclaimedSender::claim).
pub struct Sender<T> {
    inner: low_level::Sender,
    phantom: PhantomData<fn(T)>,
}

impl<T> Sender<T> {
    pub(crate) fn new(inner: low_level::Sender) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the sender into a low-level [`Sender`](low_level::Sender).
    pub fn into_lowlevel(self) -> low_level::Sender {
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
    pub fn cast<U>(self) -> Sender<U> {
        Sender::new(self.inner)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.inner.poll_close(cx)
    }

    /// Closes the sender.
    ///
    /// This will prevent further items from being sent. The receiver will still be able pull all
    /// already sent items from the channel.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle
    ///     .create_channel()
    ///     .claim_sender()
    ///     .await?;
    ///
    /// // Establish the channel:
    /// let mut receiver = receiver.claim(16).await?;
    /// let mut sender = sender.establish().await?;
    ///
    /// // Send a few items and then close the sender:
    /// sender.send_item(1).await?;
    /// sender.send_item(2).await?;
    /// sender.close().await?;
    ///
    /// // The receiver will encounter a None value after both items:
    /// assert_eq!(receiver.next_item().await?, Some(1));
    /// assert_eq!(receiver.next_item().await?, Some(2));
    /// assert_eq!(receiver.next_item().await?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Polls for whether the receiver was closed.
    ///
    /// This is different from [`poll_close`](Self::poll_close) (and [`close`](Self::close) in that
    /// this method does not close the channel from the sender's side, but merely checks whether the
    /// receiver has done so.
    pub fn poll_receiver_closed(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_receiver_closed(cx)
    }

    /// Waits until the receiver was closed.
    ///
    /// This is different from [`close`](Self::close) in that this method does not close the channel
    /// from the sender's side, but merely waits until the receiver has done so.
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
    /// // Establish the channel:
    /// let mut receiver = receiver.claim(16).await?;
    /// let mut sender = sender.establish().await?;
    ///
    /// // Close the receiver:
    /// receiver.close().await?;
    ///
    /// // The sender will notice that the receiver was closed:
    /// sender.receiver_closed().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receiver_closed(&mut self) {
        self.inner.receiver_closed().await;
    }

    /// Polls the channel for capacity to send at least one item.
    ///
    /// See [`send_ready`](Self::send_ready) for more information.
    pub fn poll_send_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.inner.poll_send_ready(cx)
    }

    /// Waits until the channel has capacity to send at least one item.
    ///
    /// Before sending an item, this method must be used to ensure there is capacity.
    pub async fn send_ready(&mut self) -> Result<(), Error> {
        self.inner.send_ready().await
    }
}

impl<T: PrimaryTag> Sender<T> {
    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_item(&mut self, item: impl Serialize<T::Tag>) -> Result<(), Error> {
        self.inner.start_send_item_as(item)
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_item`](Self::start_send_item).
    pub async fn send_item(&mut self, item: impl Serialize<T::Tag>) -> Result<(), Error> {
        self.inner.send_item_as(item).await
    }
}

impl<T: SerializePrimary> Sender<T> {
    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_item_val(&mut self, item: T) -> Result<(), Error> {
        self.start_send_item(item)
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_item_val`](Self::start_send_item_val).
    pub async fn send_item_val(&mut self, item: T) -> Result<(), Error> {
        self.send_item(item).await
    }
}

impl<'a, T> Sender<T>
where
    T: PrimaryTag + 'a,
    &'a T: Serialize<T::Tag>,
{
    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_item_ref(&mut self, item: &'a T) -> Result<(), Error> {
        self.start_send_item(item)
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_item_ref`](Self::start_send_item_ref).
    pub async fn send_item_ref(&mut self, item: &'a T) -> Result<(), Error> {
        self.send_item(item).await
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Sender")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(feature = "sink")]
impl<T: PrimaryTag, U: Serialize<T::Tag>> Sink<U> for Sender<T> {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_send_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: U) -> Result<(), Self::Error> {
        self.start_send_item(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Self::poll_close(&mut self, cx)
    }
}

/// The receiving end of an established channel.
///
/// [`Receiver`s](Self) are acquired by either
/// [`PendingReceiver::establish`](super::PendingReceiver::establish) or
/// [`UnclaimedReceiver::claim`](super::UnclaimedReceiver::claim).
pub struct Receiver<T> {
    inner: low_level::Receiver,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Receiver<T> {
    pub(crate) fn new(inner: low_level::Receiver) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Converts the receiver into a low-level [`Receiver`](low_level::Receiver).
    pub fn into_lowlevel(self) -> low_level::Receiver {
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
    pub fn cast<U>(self) -> Receiver<U> {
        Receiver::new(self.inner)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.inner.poll_close(cx)
    }

    /// Closes the receiver.
    ///
    /// This will prevent further items from being sent. The receiver will still be able pull all
    /// already sent items from the channel.
    ///
    /// Note that there might be a delay until the sender sees the channel as closed. It might still
    /// be able to send some items.
    ///
    /// ```
    /// # use aldrin::Error;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle
    ///     .create_channel()
    ///     .claim_receiver(16)
    ///     .await?;
    ///
    /// // Establish the channel:
    /// let mut sender = sender.claim().await?;
    /// let mut receiver = receiver.establish().await?;
    ///
    /// // Send a few items and then close the receiver:
    /// sender.send_item(1).await?;
    /// sender.send_item(2).await?;
    /// # handle.sync_broker().await?;
    /// receiver.close().await?;
    /// # handle.sync_broker().await?;
    ///
    /// // The receiver will encounter a None value after both items:
    /// assert_eq!(receiver.next_item().await?, Some(1));
    /// assert_eq!(receiver.next_item().await?, Some(2));
    /// assert_eq!(receiver.next_item().await?, None);
    ///
    /// // The sender will encounter an error when trying to send more items:
    /// let res = sender.send_item(3).await;
    /// assert_eq!(res, Err(Error::InvalidChannel));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }
}

impl<T: PrimaryTag> Receiver<T> {
    /// Polls the channel for the next item.
    pub fn poll_next_item_as<U: Deserialize<T::Tag>>(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<Option<U>, Error>> {
        self.inner.poll_next_item_as(cx)
    }

    /// Waits for the next item on the channel.
    pub async fn next_item_as<U: Deserialize<T::Tag>>(&mut self) -> Result<Option<U>, Error> {
        future::poll_fn(|cx| self.poll_next_item_as(cx)).await
    }
}

impl<T: DeserializePrimary> Receiver<T> {
    /// Polls the channel for the next item.
    pub fn poll_next_item(&mut self, cx: &mut Context) -> Poll<Result<Option<T>, Error>> {
        self.poll_next_item_as(cx)
    }

    /// Waits for the next item on the channel.
    pub async fn next_item(&mut self) -> Result<Option<T>, Error> {
        future::poll_fn(|cx| self.poll_next_item(cx)).await
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Receiver")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: DeserializePrimary> Stream for Receiver<T> {
    type Item = Result<T, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_next_item(cx).map(Result::transpose)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T: DeserializePrimary> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
