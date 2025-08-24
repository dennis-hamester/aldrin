use super::RawChannel;
use crate::{Error, Handle};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{ChannelCookie, Deserialize, Serialize, SerializedValue};
use futures_channel::mpsc;
use futures_core::stream::{FusedStream, Stream};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use std::future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

const LOW_CAPACITY: u32 = 4;

/// The sending end of an established channel.
///
/// [`Sender`s](Self) are acquired by either
/// [`PendingSender::establish`](super::PendingSender::establish) or
/// [`UnclaimedSender::claim`](super::UnclaimedSender::claim).
#[derive(Debug)]
pub struct Sender {
    inner: RawChannel<true>,
    capacity_added: mpsc::UnboundedReceiver<u32>,
    capacity: u32,
}

impl Sender {
    pub(crate) fn new(
        inner: RawChannel<true>,
        capacity_added: mpsc::UnboundedReceiver<u32>,
        capacity: u32,
    ) -> Self {
        Self {
            inner,
            capacity_added,
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

    /// Casts to a high-level [`Sender`](crate::Sender) by binding an item type `T`.
    pub fn cast<T>(self) -> crate::Sender<T> {
        crate::Sender::new(self)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.capacity_added.close();
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
    ///     .create_low_level_channel()
    ///     .claim_sender()
    ///     .await?;
    ///
    /// // Establish the channel:
    /// let mut receiver = receiver.claim(16).await?;
    /// let mut sender = sender.establish().await?;
    ///
    /// // Send a few items and then close the sender:
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// sender.close().await?;
    ///
    /// // The receiver will encounter a None value after both items:
    /// assert_eq!(receiver.next_item().await?, Some(1));
    /// assert_eq!(receiver.next_item().await?, Some(2));
    /// assert_eq!(receiver.next_item::<i32>().await?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    /// Polls for whether the receiver was closed.
    ///
    /// This is different from [`poll_close`](Self::poll_close) (and [`close`](Self::close) in that
    /// this method does not close the channel from the sender's side, but merely checks whether the
    /// receiver has done so.
    pub fn poll_receiver_closed(&mut self, cx: &mut Context) -> Poll<()> {
        loop {
            match Pin::new(&mut self.capacity_added).poll_next(cx) {
                Poll::Ready(Some(added_capacity)) => self.capacity += added_capacity,
                Poll::Ready(None) => break Poll::Ready(()),
                Poll::Pending => break Poll::Pending,
            }
        }
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
    ///     .create_low_level_channel()
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
        future::poll_fn(|cx| self.poll_receiver_closed(cx)).await
    }

    /// Polls the channel for capacity to send at least one item.
    ///
    /// See [`send_ready`](Self::send_ready) for more information.
    pub fn poll_send_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        loop {
            match Pin::new(&mut self.capacity_added).poll_next(cx) {
                Poll::Ready(Some(added_capacity)) => self.capacity += added_capacity,
                Poll::Ready(None) => break Poll::Ready(Err(Error::InvalidChannel)),
                Poll::Pending if self.capacity > 0 => break Poll::Ready(Ok(())),
                Poll::Pending => break Poll::Pending,
            }
        }
    }

    /// Waits until the channel has capacity to send at least one item.
    ///
    /// Before sending an item, this method must be used to ensure there is capacity.
    pub async fn send_ready(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_send_ready(cx)).await
    }

    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_serialized(&mut self, item: SerializedValue) -> Result<(), Error> {
        debug_assert!(self.capacity > 0);
        self.inner.send_item(item)?;
        self.capacity -= 1;
        Ok(())
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_serialized`](Self::start_send_serialized).
    pub async fn send_serialized(&mut self, item: SerializedValue) -> Result<(), Error> {
        self.send_ready().await?;
        self.start_send_serialized(item)
    }

    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_item_as<T: Tag>(&mut self, item: impl Serialize<T>) -> Result<(), Error> {
        let item = SerializedValue::serialize_as(item)?;
        self.start_send_serialized(item)
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_item_as`](Self::start_send_item_as).
    pub async fn send_item_as<T: Tag>(&mut self, item: impl Serialize<T>) -> Result<(), Error> {
        self.send_ready().await?;
        self.start_send_item_as(item)
    }

    /// Starts sending an item on the channel.
    ///
    /// It must be ensured that there is enough capacity by calling [`send_ready`](Self::send_ready)
    /// prior to sending an item.
    pub fn start_send_item<T: PrimaryTag + Serialize<T::Tag>>(
        &mut self,
        item: T,
    ) -> Result<(), Error> {
        self.start_send_item_as(item)
    }

    /// Sends an item on the channel.
    ///
    /// This method is a shorthand for calling [`send_ready`](Self::send_ready) followed by
    /// [`start_send_item`](Self::start_send_item).
    pub async fn send_item<T: PrimaryTag + Serialize<T::Tag>>(
        &mut self,
        item: T,
    ) -> Result<(), Error> {
        self.send_ready().await?;
        self.start_send_item(item)
    }
}

#[cfg(feature = "sink")]
impl Sink<SerializedValue> for Sender {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_send_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: SerializedValue) -> Result<(), Self::Error> {
        self.start_send_serialized(item)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.inner.is_open() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::InvalidChannel))
        }
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
#[derive(Debug)]
pub struct Receiver {
    inner: RawChannel<false>,
    items: mpsc::UnboundedReceiver<SerializedValue>,
    max_capacity: NonZeroU32,
    cur_capacity: u32,
}

impl Receiver {
    pub(crate) fn new(
        inner: RawChannel<false>,
        items: mpsc::UnboundedReceiver<SerializedValue>,
        max_capacity: NonZeroU32,
    ) -> Self {
        Self {
            inner,
            items,
            max_capacity,
            cur_capacity: max_capacity.get(),
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

    /// Casts to a high-level [`Receiver`](crate::Receiver) by binding an item type `T`.
    pub fn cast<T>(self) -> crate::Receiver<T> {
        crate::Receiver::new(self)
    }

    /// Initiates closing the sender and polls for progress.
    ///
    /// See [`close`](Self::close) for more information.
    pub fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.items.close();
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
    ///     .create_low_level_channel()
    ///     .claim_receiver(16)
    ///     .await?;
    ///
    /// // Establish the channel:
    /// let mut sender = sender.claim().await?;
    /// let mut receiver = receiver.establish().await?;
    ///
    /// // Send a few items and then close the receiver:
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// # handle.sync_broker().await?;
    /// receiver.close().await?;
    /// # handle.sync_broker().await?;
    ///
    /// // The receiver will encounter a None value after both items:
    /// assert_eq!(receiver.next_item().await?, Some(1));
    /// assert_eq!(receiver.next_item().await?, Some(2));
    /// assert_eq!(receiver.next_item::<i32>().await?, None);
    ///
    /// // The sender will encounter an error when trying to send more items:
    /// let res = sender.send_item(&3).await;
    /// assert_eq!(res, Err(Error::InvalidChannel));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_close(cx)).await
    }

    /// Polls the channel for the next item.
    pub fn poll_next_serialized(&mut self, cx: &mut Context) -> Poll<Option<SerializedValue>> {
        debug_assert!(self.cur_capacity > 0);
        debug_assert!(self.cur_capacity <= self.max_capacity.get());

        let item = match Pin::new(&mut self.items).poll_next(cx) {
            Poll::Ready(Some(item)) => item,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
        };

        self.cur_capacity -= 1;
        if self.cur_capacity <= LOW_CAPACITY {
            let diff = self.max_capacity.get() - self.cur_capacity;
            debug_assert!(diff >= 1);

            self.inner.add_channel_capacity(diff);
            self.cur_capacity += diff;
        }

        debug_assert!(self.cur_capacity > 0);
        debug_assert!(self.cur_capacity <= self.max_capacity.get());

        Poll::Ready(Some(item))
    }

    /// Waits for the next item on the channel.
    pub async fn next_serialized(&mut self) -> Option<SerializedValue> {
        future::poll_fn(|cx| self.poll_next_serialized(cx)).await
    }

    /// Polls the channel for the next item.
    pub fn poll_next_item_as<T: Tag, U: Deserialize<T>>(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<Option<U>, Error>> {
        match self.poll_next_serialized(cx) {
            Poll::Ready(Some(item)) => {
                Poll::Ready(item.deserialize_as().map(Some).map_err(Error::invalid_item))
            }

            Poll::Ready(None) => Poll::Ready(Ok(None)),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Waits for the next item on the channel.
    pub async fn next_item_as<T: Tag, U: Deserialize<T>>(&mut self) -> Result<Option<U>, Error> {
        future::poll_fn(|cx| self.poll_next_item_as(cx)).await
    }

    /// Polls the channel for the next item.
    pub fn poll_next_item<T: PrimaryTag + Deserialize<T::Tag>>(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<Option<T>, Error>> {
        self.poll_next_item_as(cx)
    }

    /// Waits for the next item on the channel.
    pub async fn next_item<T: PrimaryTag + Deserialize<T::Tag>>(
        &mut self,
    ) -> Result<Option<T>, Error> {
        future::poll_fn(|cx| self.poll_next_item(cx)).await
    }
}

impl Stream for Receiver {
    type Item = SerializedValue;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_next_serialized(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl FusedStream for Receiver {
    fn is_terminated(&self) -> bool {
        self.items.is_terminated()
    }
}
