#[cfg(test)]
mod test;

use super::Handle;
#[cfg(feature = "introspection")]
use crate::core::introspection::{
    BuiltInType, DynIntrospectable, Introspectable, Layout, LexicalId,
};
use crate::core::{
    ChannelCookie, ChannelEnd, Deserialize, DeserializeError, Deserializer, Serialize,
    SerializeError, SerializedValue, Serializer,
};
use crate::error::Error;
#[cfg(feature = "sink")]
use crate::handle::CloseChannelEndFuture;
use futures_channel::{mpsc, oneshot};
use futures_core::stream::{FusedStream, Stream};
use std::fmt;
use std::future;
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

const LOW_CAPACITY: u32 = 4;

/// A sender that is not bound to any particular client.
///
/// [`UnboundSender`s](Self) are used to transfer senders to some other client, typically by
/// returning them from function calls.
///
/// When [creating a channel](Handle::create_channel_with_claimed_receiver) the resulting
/// [`UnclaimedSender`] can be [unbound](UnclaimedSender::unbind) and sent to another client.
///
/// It is worth noting that this type implements [`Copy`] and [`Clone`]. As such (and because it is
/// not bound to any client), it will not close the sending end of a channel. This is the main
/// difference from `UnclaimedSender`.
pub struct UnboundSender<T: ?Sized> {
    cookie: ChannelCookie,
    phantom: PhantomData<fn(T)>,
}

impl<T: ?Sized> UnboundSender<T> {
    /// Creates a new [`UnboundSender`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self {
            cookie,
            phantom: PhantomData,
        }
    }

    /// Returns the sender's [`ChannelCookie`].
    pub fn cookie(self) -> ChannelCookie {
        self.cookie
    }

    /// Binds the sender to a client.
    ///
    /// See also [`claim`](Self::claim) to bind and claim the sender in one step.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::{UnclaimedSender, Sender};
    ///
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>(16).await?;
    /// # let sender = sender.unbind();
    /// // Assume this sender has been returned from some function call.
    /// // let sender: UnboundSender<u32> = ...
    ///
    /// // Bind it to the local client. The explicit type is shown here only for the sake of the
    /// // example.
    /// let sender: UnclaimedSender<u32> = sender.bind(handle.clone());
    ///
    /// // Afterwards, it can be claimed.
    /// let sender: Sender<u32> = sender.claim().await?;
    /// # Ok(())
    /// # }
    pub fn bind(self, client: Handle) -> UnclaimedSender<T> {
        UnclaimedSender::new(UnclaimedSenderInner::new(self.cookie, client))
    }

    /// Binds the sender to a client and claims it.
    ///
    /// This function is equivalent to `sender.bind(client).claim()`.
    ///
    /// See [`UnclaimedSender::claim`] for explanation of the cases in which this function can fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::Sender;
    ///
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>(16).await?;
    /// # let sender = sender.unbind();
    /// // Assume this sender has been returned from some function call.
    /// // let sender: UnboundSender<u32> = ...
    ///
    /// // Bind it to the local client and claim it, so that it can immediately be used. The
    /// // explicit type here is given only for the sake of the example.
    /// let sender: Sender<u32> = sender.claim(handle.clone()).await?;
    /// # Ok(())
    /// # }
    pub async fn claim(self, client: Handle) -> Result<Sender<T>, Error> {
        self.bind(client).claim().await
    }

    /// Casts the item type to a different type.
    pub fn cast<U: ?Sized>(self) -> UnboundSender<U> {
        UnboundSender {
            cookie: self.cookie,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> fmt::Debug for UnboundSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnboundSender")
            .field("cookie", &self.cookie)
            .finish()
    }
}

impl<T: ?Sized> Clone for UnboundSender<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for UnboundSender<T> {}

impl<T1: ?Sized, T2: ?Sized> PartialEq<UnboundSender<T2>> for UnboundSender<T1> {
    fn eq(&self, rhs: &UnboundSender<T2>) -> bool {
        self.cookie == rhs.cookie
    }
}

impl<T: ?Sized> Eq for UnboundSender<T> {}

impl<T: ?Sized> Serialize for UnboundSender<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(self.cookie);
        Ok(())
    }
}

impl<T: ?Sized> Deserialize for UnboundSender<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(Self::new)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable + ?Sized> Introspectable for UnboundSender<T> {
    fn layout() -> Layout {
        BuiltInType::Sender(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::sender(T::lexical_id())
    }

    fn inner_types(types: &mut Vec<DynIntrospectable>) {
        types.push(DynIntrospectable::new::<T>());
    }
}

/// A sender that hasn't been claimed yet.
///
/// [`UnclaimedSender`s](Self) are similar to [`UnboundSender`s](UnboundSender) in that they
/// identify the sending end of a channel in an unclaimed state. This sender is however bound to a
/// client and can thus be claimed.
pub struct UnclaimedSender<T: ?Sized> {
    inner: UnclaimedSenderInner,
    phantom: PhantomData<fn(T)>,
}

impl<T: ?Sized> UnclaimedSender<T> {
    pub(crate) fn new(inner: UnclaimedSenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the sender's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the sender.
    ///
    /// `None` will be returned if the sender is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Unbinds the sender from its client.
    ///
    /// When creating a channel, one end will already be claimed while the other end won't. In order
    /// to send that other end (here the sender) to another client, it must first be unbound from
    /// its client.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create a channel with an unclaimed sender and a claimed receiver.
    /// let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>(16).await?;
    ///
    /// // Unbind the sender so that it can be sent to another client. This will typically happen by
    /// // returning it from a function call.
    /// let sender = sender.unbind();
    /// # Ok(())
    /// # }
    pub fn unbind(self) -> UnboundSender<T> {
        UnboundSender::new(self.inner.unbind())
    }

    /// Closes the sender without consuming it.
    ///
    /// This closes the sender such that it cannot be claimed anymore by any client. When the
    /// receiver waits for the channel to become [established](PendingReceiver::established), an
    /// error will be returned.
    ///
    /// After closing a sender, any further function calls will return [`Error::InvalidChannel`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::Error;
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>(16).await?;
    ///
    /// // Close the sender.
    /// sender.close().await?;
    ///
    /// // For the receiver, an error will be returned when waiting for the channel to become
    /// // established.
    /// let err = receiver.established().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Claims the sender by its bound client.
    ///
    /// When creating channels, both ends must be claimed by a client before items can be sent and
    /// received. One end will always be claimed automatically. The other end, here the sender, must
    /// be claimed manually (after it has possibly been sent to another client).
    ///
    /// When this function returns successfully, a receiver's call to
    /// [`PendingReceiver::established`] will also resolve successfully.
    ///
    /// This function can fail in the following cases:
    /// - Some other client has already claimed the sender.
    /// - Some other client has closed the sender.
    /// - The receiver has been closed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // The sender is unclaimed, while the receiver has been claimed automatically.
    /// let (sender, receiver) = handle.create_channel_with_claimed_receiver(16).await?;
    ///
    /// // Claim the sender.
    /// let mut sender = sender.claim().await?;
    ///
    /// // This will now resolve immediately.
    /// let mut receiver = receiver.established().await?;
    ///
    /// // The channel is now fully established and items can be sent and received.
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// assert_eq!(receiver.next_item().await, Ok(Some(1)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(2)));
    /// # Ok(())
    /// # }
    pub async fn claim(self) -> Result<Sender<T>, Error> {
        self.inner.claim().await.map(Sender::new)
    }

    /// Casts the item type to a different type.
    pub fn cast<U: ?Sized>(self) -> UnclaimedSender<U> {
        UnclaimedSender {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> fmt::Debug for UnclaimedSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnclaimedSender")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct UnclaimedSenderInner {
    cookie: ChannelCookie,
    client: Option<Handle>,
}

impl UnclaimedSenderInner {
    pub(crate) fn new(cookie: ChannelCookie, client: Handle) -> Self {
        Self {
            cookie,
            client: Some(client),
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        self.client.as_ref()
    }

    fn unbind(mut self) -> ChannelCookie {
        self.client = None;
        self.cookie
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let Some(client) = self.client.take() {
            client
                .close_channel_end(self.cookie, ChannelEnd::Sender, false)?
                .await
        } else {
            Ok(())
        }
    }

    async fn claim(mut self) -> Result<SenderInner, Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client.claim_sender(self.cookie).await
    }
}

impl Drop for UnclaimedSenderInner {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.close_channel_end(self.cookie, ChannelEnd::Sender, false);
        }
    }
}

/// A claimed sender that is waiting for the channel to become established.
///
/// [`PendingSender`s](Self) are used to wait until some client has claimed the receiving end of the
/// channel. This is done with the [`established`](Self::established) function.
pub struct PendingSender<T: ?Sized> {
    inner: PendingSenderInner,
    phantom: PhantomData<fn(T)>,
}

impl<T: ?Sized> PendingSender<T> {
    pub(crate) fn new(inner: PendingSenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the sender's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the sender.
    ///
    /// `None` will be returned if the sender is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Closes the sender without consuming it.
    ///
    /// When closing a [`PendingSender`], it will no longer be possible to claim the receiver. If it
    /// has already been claimed, then it will receive `None`, indicating that the channel has been
    /// closed.
    ///
    /// # Examples
    ///
    /// ## Closing a sender while the receiver hasn't been claimed yet
    ///
    /// ```
    /// use aldrin::Error;
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Close the sender.
    /// sender.close().await?;
    ///
    /// // For the receiver, an error will be returned when trying to claim it.
    /// let err = receiver.claim(16).await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Closing a sender while the receiver has already been claimed
    ///
    /// ```
    /// use aldrin::Error;
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Claim the receiver.
    /// let mut receiver = receiver.claim(16).await?;
    ///
    /// // Close the sender.
    /// sender.close().await?;
    ///
    /// // The receiver will receive None.
    /// assert_eq!(receiver.next_item().await, Ok(None));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Waits until the channel has been established.
    ///
    /// A channel is established when both ends have been claimed. An error is returned when the
    /// receiver has been closed instead of claimed.
    pub async fn established(self) -> Result<Sender<T>, Error> {
        self.inner.established().await.map(Sender::new)
    }

    /// Casts the item type to a different type.
    pub fn cast<U: ?Sized>(self) -> PendingSender<U> {
        PendingSender {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> fmt::Debug for PendingSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingSender")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct PendingSenderInner {
    cookie: ChannelCookie,
    state: Option<PendingSenderInnerState>,
}

#[derive(Debug)]
struct PendingSenderInnerState {
    client: Handle,
    established: oneshot::Receiver<(u32, mpsc::UnboundedReceiver<u32>)>,
}

impl PendingSenderInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        established: oneshot::Receiver<(u32, mpsc::UnboundedReceiver<u32>)>,
    ) -> Self {
        Self {
            cookie,
            state: Some(PendingSenderInnerState {
                client,
                established,
            }),
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        self.state.as_ref().map(|state| &state.client)
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let Some(state) = self.state.take() {
            state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Sender, true)?
                .await
        } else {
            Ok(())
        }
    }

    async fn established(mut self) -> Result<SenderInner, Error> {
        let state = self.state.take().ok_or(Error::InvalidChannel)?;
        let client = state.client;

        state
            .established
            .await
            .map(|(capacity, capacity_added)| {
                SenderInner::new(self.cookie, client, capacity_added, capacity)
            })
            .map_err(|_| Error::InvalidChannel)
    }
}

impl Drop for PendingSenderInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            let _ = state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Sender, true);
        }
    }
}

/// The sending end of an established channel.
///
/// This type of sender is obtained when a channel has been fully established, either by
/// [`PendingSender::established`] or by [`UnclaimedSender::claim`].
pub struct Sender<T: ?Sized> {
    inner: SenderInner,
    phantom: PhantomData<fn(T)>,
}

impl<T: ?Sized> Sender<T> {
    pub(crate) fn new(inner: SenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the sender's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the sender.
    ///
    /// `None` will be returned if the sender is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Casts the item type to a different type.
    pub fn cast<U: ?Sized>(self) -> Sender<U> {
        Sender {
            inner: self.inner,
            phantom: PhantomData,
        }
    }

    /// Polls the channel for capacity to send at least 1 item.
    pub fn poll_send_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.inner.poll_send_ready(cx)
    }

    /// Waits until the channel has capacity to send at least 1 item.
    pub async fn send_ready(&mut self) -> Result<(), Error> {
        future::poll_fn(|cx| self.poll_send_ready(cx)).await
    }

    /// Closes the sender without consuming it.
    ///
    /// The will cause the receiving end to receive [`None`] after all other items have been
    /// received.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle.create_channel_with_claimed_sender().await?;
    ///
    /// let mut receiver = receiver.claim(16).await?;
    /// let mut sender = sender.established().await?;
    ///
    /// // Send a couple of items and then close the sender.
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// sender.send_item(&3).await?;
    /// sender.close().await?;
    ///
    /// // The receiver will receive all items followed by None.
    /// assert_eq!(receiver.next_item().await, Ok(Some(1)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(2)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(3)));
    /// assert_eq!(receiver.next_item().await, Ok(None));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Polls whether the channel is closed.
    pub fn poll_closed(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_closed(cx)
    }

    /// Completes when the channel is closed.
    pub async fn closed(&mut self) {
        future::poll_fn(|cx| self.poll_closed(cx)).await
    }
}

impl<T: Serialize + ?Sized> Sender<T> {
    /// Sends an item on the channel.
    ///
    /// This function panics if the channel doesn't have any capacity left. You must call either
    /// [`send_ready`](Self::send_ready) or [`poll_send_ready`](Self::poll_send_ready) before to
    /// ensure there is capacity.
    pub fn start_send_item(&mut self, item: &T) -> Result<(), Error> {
        self.inner.start_send_item(item)
    }

    /// Sends an item on the channel.
    ///
    /// This function will wait until the channel has capacity to send at least 1 item.
    pub async fn send_item(&mut self, item: &T) -> Result<(), Error> {
        self.send_ready().await?;
        self.start_send_item(item)
    }
}

impl<T: ?Sized> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Sender")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(feature = "sink")]
impl<T: Serialize + ?Sized> futures_sink::Sink<&T> for Sender<T> {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_send_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: &T) -> Result<(), Self::Error> {
        self.start_send_item(item)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_flush()
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_close(cx)
    }
}

#[cfg(feature = "sink")]
impl<T: Serialize> futures_sink::Sink<T> for Sender<T> {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_send_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.start_send_item(&item)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_flush()
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_close(cx)
    }
}

#[derive(Debug)]
pub(crate) struct SenderInner {
    cookie: ChannelCookie,
    state: SenderInnerState,
}

#[derive(Debug)]
enum SenderInnerState {
    Open {
        client: Handle,
        capacity_added: mpsc::UnboundedReceiver<u32>,
        capacity: u32,
    },

    Closed,

    #[cfg(feature = "sink")]
    Closing(CloseChannelEndFuture),
}

impl SenderInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        capacity_added: mpsc::UnboundedReceiver<u32>,
        capacity: u32,
    ) -> Self {
        Self {
            cookie,
            state: SenderInnerState::Open {
                client,
                capacity_added,
                capacity,
            },
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        match self.state {
            SenderInnerState::Open { ref client, .. } => Some(client),
            SenderInnerState::Closed => None,

            #[cfg(feature = "sink")]
            SenderInnerState::Closing(_) => None,
        }
    }

    fn poll_send_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        let SenderInnerState::Open {
            ref mut capacity_added,
            ref mut capacity,
            ..
        } = self.state
        else {
            return Poll::Ready(Err(Error::InvalidChannel));
        };

        if *capacity == 0 {
            loop {
                match Pin::new(&mut *capacity_added).poll_next(cx) {
                    Poll::Ready(Some(added_capacity)) => *capacity += added_capacity,

                    Poll::Ready(None) => {
                        self.state = SenderInnerState::Closed;
                        return Poll::Ready(Err(Error::InvalidChannel));
                    }

                    Poll::Pending => {
                        if *capacity > 0 {
                            break;
                        } else {
                            return Poll::Pending;
                        }
                    }
                }
            }
        } else {
            match capacity_added.try_next() {
                Ok(Some(added_capacity)) => *capacity += added_capacity,

                Ok(None) => {
                    self.state = SenderInnerState::Closed;
                    return Poll::Ready(Err(Error::InvalidChannel));
                }

                Err(_) => {}
            }
        }

        Poll::Ready(Ok(()))
    }

    fn start_send_item<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        let SenderInnerState::Open {
            ref client,
            ref mut capacity,
            ..
        } = self.state
        else {
            return Err(Error::InvalidChannel);
        };

        debug_assert!(*capacity > 0);

        let value = SerializedValue::serialize(value)?;
        client.send_item(self.cookie, value)?;

        *capacity -= 1;

        Ok(())
    }

    #[cfg(feature = "sink")]
    fn poll_flush(&self) -> Poll<Result<(), Error>> {
        if let SenderInnerState::Open { .. } = self.state {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::InvalidChannel))
        }
    }

    #[cfg(feature = "sink")]
    fn poll_close(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        use std::future::Future;

        let mut fut = match mem::replace(&mut self.state, SenderInnerState::Closed) {
            SenderInnerState::Open { client, .. } => {
                match client.close_channel_end(self.cookie, ChannelEnd::Sender, true) {
                    Ok(fut) => fut,
                    Err(e) => return Poll::Ready(Err(e)),
                }
            }

            SenderInnerState::Closing(fut) => fut,
            SenderInnerState::Closed => return Poll::Ready(Ok(())),
        };

        let res = Pin::new(&mut fut).poll(cx);

        if res.is_pending() {
            self.state = SenderInnerState::Closing(fut);
        }

        res
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let SenderInnerState::Open { client, .. } =
            mem::replace(&mut self.state, SenderInnerState::Closed)
        {
            client
                .close_channel_end(self.cookie, ChannelEnd::Sender, true)?
                .await
        } else {
            Ok(())
        }
    }

    fn is_closed(&self) -> bool {
        matches!(self.state, SenderInnerState::Closed)
    }

    fn poll_closed(&mut self, cx: &mut Context) -> Poll<()> {
        let (capacity_added, capacity) = match self.state {
            SenderInnerState::Open {
                ref mut capacity_added,
                ref mut capacity,
                ..
            } => (capacity_added, capacity),

            SenderInnerState::Closed => return Poll::Ready(()),

            #[cfg(feature = "sink")]
            SenderInnerState::Closing(_) => return self.poll_close(cx).map(|_| ()),
        };

        loop {
            match Pin::new(&mut *capacity_added).poll_next(cx) {
                Poll::Ready(Some(added_capacity)) => *capacity += added_capacity,

                Poll::Ready(None) => {
                    self.state = SenderInnerState::Closed;
                    break Poll::Ready(());
                }

                Poll::Pending => break Poll::Pending,
            }
        }
    }
}

impl Drop for SenderInner {
    fn drop(&mut self) {
        if let SenderInnerState::Open { client, .. } =
            mem::replace(&mut self.state, SenderInnerState::Closed)
        {
            let _ = client.close_channel_end(self.cookie, ChannelEnd::Sender, true);
        }
    }
}

/// A receiver that is not bound to any particular client.
///
/// [`UnboundReceiver`s](Self) are used to transfer receivers to some other client, typically by
/// returning them from function calls.
///
/// When [creating a channel](Handle::create_channel_with_claimed_sender) the resulting
/// [`UnclaimedReceiver`] can be [unbound](UnclaimedReceiver::unbind) and sent to another client.
///
/// It is worth noting that this type implements [`Copy`] and [`Clone`]. As such (and because it is
/// not bound to any client), it will not close the receiving end of a channel. This is the main
/// difference from `UnclaimedReceiver`.
pub struct UnboundReceiver<T> {
    cookie: ChannelCookie,
    phantom: PhantomData<fn() -> T>,
}

impl<T> UnboundReceiver<T> {
    /// Creates a new [`UnboundReceiver`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self {
            cookie,
            phantom: PhantomData,
        }
    }

    /// Returns the receiver's [`ChannelCookie`].
    pub fn cookie(self) -> ChannelCookie {
        self.cookie
    }

    /// Binds the receiver to a client.
    ///
    /// See also [`claim`](Self::claim) to bind and claim the receiver in one step.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::{Receiver, UnclaimedReceiver};
    ///
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume this receiver has been returned from some function call.
    /// // let receiver: UnboundReceiver<u32> = ...
    ///
    /// // Bind it to the local client. The explicit type is shown here only for the sake of the
    /// // example.
    /// let receiver: UnclaimedReceiver<u32> = receiver.bind(handle.clone());
    ///
    /// // Afterwards, it can be claimed.
    /// let receiver: Receiver<u32> = receiver.claim(16).await?;
    /// # Ok(())
    /// # }
    pub fn bind(self, client: Handle) -> UnclaimedReceiver<T> {
        UnclaimedReceiver::new(UnclaimedReceiverInner::new(self.cookie, client))
    }

    /// Binds the receiver to a client and claims it.
    ///
    /// This function is equivalent to `receiver.bind(client).claim()`.
    ///
    /// See [`UnclaimedReceiver::claim`] for explanation of the cases in which this function can
    /// fail.
    ///
    /// A `capacity` of 0 is treated as if 1 was specificed instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::Receiver;
    ///
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume this receiver has been returned from some function call.
    /// // let receiver: UnboundReceiver<u32> = ...
    ///
    /// // Bind it to the local client and claim it, so that it can immediately be used. The
    /// // explicit type here is given only for the sake of the example.
    /// let receiver: Receiver<u32> = receiver.claim(handle.clone(), 16).await?;
    /// # Ok(())
    /// # }
    pub async fn claim(self, client: Handle, capacity: u32) -> Result<Receiver<T>, Error> {
        self.bind(client).claim(capacity).await
    }

    /// Casts the item type to a different type.
    pub fn cast<U>(self) -> UnboundReceiver<U> {
        UnboundReceiver {
            cookie: self.cookie,
            phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for UnboundReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnboundReceiver")
            .field("cookie", &self.cookie)
            .finish()
    }
}

impl<T> Clone for UnboundReceiver<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnboundReceiver<T> {}

impl<T1, T2> PartialEq<UnboundReceiver<T2>> for UnboundReceiver<T1> {
    fn eq(&self, rhs: &UnboundReceiver<T2>) -> bool {
        self.cookie == rhs.cookie
    }
}

impl<T> Eq for UnboundReceiver<T> {}

impl<T> Serialize for UnboundReceiver<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(self.cookie);
        Ok(())
    }
}

impl<T> Deserialize for UnboundReceiver<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(Self::new)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for UnboundReceiver<T> {
    fn layout() -> Layout {
        BuiltInType::Receiver(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::receiver(T::lexical_id())
    }

    fn inner_types(types: &mut Vec<DynIntrospectable>) {
        types.push(DynIntrospectable::new::<T>());
    }
}

/// A receiver that hasn't been claimed yet.
///
/// [`UnclaimedReceiver`s](Self) are similar to [`UnboundReceiver`s](UnboundReceiver) in that they
/// identify the receiving end of a channel in an unclaimed state. This receiver is however bound to
/// a client and can thus be claimed.
pub struct UnclaimedReceiver<T> {
    inner: UnclaimedReceiverInner,
    phantom: PhantomData<fn() -> T>,
}

impl<T> UnclaimedReceiver<T> {
    pub(crate) fn new(inner: UnclaimedReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the receiver's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the receiver.
    ///
    /// `None` will be returned if the receiver is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Unbinds the receiver from its client.
    ///
    /// When creating a channel, one end will already be claimed while the other end won't. In order
    /// to send that other end (here the receiver) to another client, it must first be unbound from
    /// its client.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create a channel with an unclaimed receiver and a claimed sender.
    /// let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Unbind the receiver so that it can be sent to another client. This will typically happen
    /// // by returning it from a function call.
    /// let receiver = receiver.unbind();
    /// # Ok(())
    /// # }
    pub fn unbind(self) -> UnboundReceiver<T> {
        UnboundReceiver::new(self.inner.unbind())
    }

    /// Closes the receiver without consuming it.
    ///
    /// This closes the receiver such that it cannot be claimed anymore by any client. When the
    /// sender waits for the channel to become [established](PendingSender::established), an error
    /// will be returned.
    ///
    /// After closing a receiver, any further function calls will return [`Error::InvalidChannel`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin::Error;
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Close the receiver.
    /// receiver.close().await?;
    ///
    /// // For the sender, an error will be returned when waiting for the channel to become
    /// // established.
    /// let err = sender.established().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Claims the receiver by its bound client.
    ///
    /// When creating channels, both ends must be claimed by a client before items can be sent and
    /// received. One end will always be claimed automatically. The other end, here the receiver,
    /// must be claimed manually (after it has possibly been sent to another client).
    ///
    /// When this function returns successfully, a senders's call to [`PendingSender::established`]
    /// will also resolve successfully.
    ///
    /// This function can fail in the following cases:
    /// - Some other client has already claimed the receiver.
    /// - Some other client has closed the receiver.
    /// - The sender has been closed.
    ///
    /// A `capacity` of 0 is treated as if 1 was specificed instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // The receiver is unclaimed, while the sender has been claimed automatically.
    /// let (sender, receiver) = handle.create_channel_with_claimed_sender().await?;
    ///
    /// // Claim the receiver.
    /// let mut receiver = receiver.claim(16).await?;
    ///
    /// // This will now resolve immediately.
    /// let mut sender = sender.established().await?;
    ///
    /// // The channel is now fully established and items can be sent and received.
    /// sender.send_item(&1).await?;
    /// sender.send_item(&2).await?;
    /// assert_eq!(receiver.next_item().await, Ok(Some(1)));
    /// assert_eq!(receiver.next_item().await, Ok(Some(2)));
    /// # Ok(())
    /// # }
    pub async fn claim(self, capacity: u32) -> Result<Receiver<T>, Error> {
        self.inner.claim(capacity).await.map(Receiver::new)
    }

    /// Casts the item type to a different type.
    pub fn cast<U>(self) -> UnclaimedReceiver<U> {
        UnclaimedReceiver {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for UnclaimedReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnclaimedReceiver")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct UnclaimedReceiverInner {
    cookie: ChannelCookie,
    client: Option<Handle>,
}

impl UnclaimedReceiverInner {
    pub(crate) fn new(cookie: ChannelCookie, client: Handle) -> Self {
        Self {
            cookie,
            client: Some(client),
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        self.client.as_ref()
    }

    fn unbind(mut self) -> ChannelCookie {
        self.client = None;
        self.cookie
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let Some(client) = self.client.take() {
            client
                .close_channel_end(self.cookie, ChannelEnd::Receiver, false)?
                .await
        } else {
            Ok(())
        }
    }

    async fn claim(mut self, capacity: u32) -> Result<ReceiverInner, Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client.claim_receiver(self.cookie, capacity).await
    }
}

impl Drop for UnclaimedReceiverInner {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.close_channel_end(self.cookie, ChannelEnd::Receiver, false);
        }
    }
}

/// A claimed receiver that is waiting for the channel to become established.
///
/// [`PendingReceiver`s](Self) are used to wait until some client has claimed the sending end of the
/// channel. This is done with the [`established`](Self::established) function.
pub struct PendingReceiver<T> {
    inner: PendingReceiverInner,
    phantom: PhantomData<fn() -> T>,
}

impl<T> PendingReceiver<T> {
    pub(crate) fn new(inner: PendingReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the receiver's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the receiver.
    ///
    /// `None` will be returned if the receiver is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Closes the receiver without consuming it.
    ///
    /// When closing a [`PendingReceiver`], it will no longer be possible to claim the sender.
    ///
    /// When the sender has already been claimed, the situation is a little bit more
    /// complicated. The sender is notified asynchronously about the receiver's closing. It will,
    /// sooner or later, receive an error when sending an item.
    ///
    /// # Examples
    ///
    /// ## Closing a receiver while the sender hasn't been claimed yet
    ///
    /// ```
    /// use aldrin::Error;
    /// # use aldrin_test::tokio::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle.create_channel_with_claimed_receiver::<u32>(16).await?;
    ///
    /// // Close the receiver.
    /// receiver.close().await?;
    ///
    /// // For the sender, an error will be returned when trying to claim it.
    /// let err = sender.claim().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Waits until the channel has been established.
    ///
    /// A channel is established when both ends have been claimed. An error is returned when the
    /// sender has been closed instead of claimed.
    pub async fn established(self) -> Result<Receiver<T>, Error> {
        self.inner.established().await.map(Receiver::new)
    }

    /// Casts the item type to a different type.
    pub fn cast<U>(self) -> PendingReceiver<U> {
        PendingReceiver {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for PendingReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PendingReceiver")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct PendingReceiverInner {
    cookie: ChannelCookie,
    state: Option<PendingReceiverInnerState>,
}

#[derive(Debug)]
struct PendingReceiverInnerState {
    client: Handle,
    established: oneshot::Receiver<mpsc::UnboundedReceiver<SerializedValue>>,
    max_capacity: NonZeroU32,
}

impl PendingReceiverInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        established: oneshot::Receiver<mpsc::UnboundedReceiver<SerializedValue>>,
        max_capacity: NonZeroU32,
    ) -> Self {
        Self {
            cookie,
            state: Some(PendingReceiverInnerState {
                client,
                established,
                max_capacity,
            }),
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        self.state.as_ref().map(|state| &state.client)
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let Some(state) = self.state.take() {
            state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Receiver, true)?
                .await
        } else {
            Ok(())
        }
    }

    async fn established(mut self) -> Result<ReceiverInner, Error> {
        let state = self.state.take().ok_or(Error::InvalidChannel)?;
        let client = state.client;

        state
            .established
            .await
            .map(|items| ReceiverInner::new(self.cookie, client, items, state.max_capacity))
            .map_err(|_| Error::InvalidChannel)
    }
}

impl Drop for PendingReceiverInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            let _ = state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Receiver, true);
        }
    }
}

/// The receiving end of an established channel.
///
/// This type of receiver is obtained when a channel has been fully established, either by
/// [`PendingReceiver::established`] or by [`UnclaimedReceiver::claim`].
pub struct Receiver<T> {
    inner: ReceiverInner,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Receiver<T> {
    pub(crate) fn new(inner: ReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the receiver's [`ChannelCookie`].
    pub fn cookie(&self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Returns a handle to the client that was used to create the receiver.
    ///
    /// `None` will be returned if the receiver is closed.
    pub fn client(&self) -> Option<&Handle> {
        self.inner.client()
    }

    /// Closes the receiver without consuming it.
    pub async fn close(&mut self) -> Result<(), Error> {
        self.inner.close().await
    }

    /// Casts the item type to a different type.
    pub fn cast<U>(self) -> Receiver<U> {
        Receiver {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T: Deserialize> Receiver<T> {
    /// Polls for the next item.
    pub fn poll_next_item(&mut self, cx: &mut Context) -> Poll<Result<Option<T>, Error>> {
        match self.inner.poll_next_item(cx) {
            Poll::Ready(Some(value)) => {
                Poll::Ready(value.deserialize().map(Some).map_err(Error::invalid_item))
            }
            Poll::Ready(None) => Poll::Ready(Ok(None)),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Returns the next item.
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

impl<T: Deserialize> Stream for Receiver<T> {
    type Item = Result<T, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_next_item(cx).map(Result::transpose)
    }
}

impl<T: Deserialize> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[derive(Debug)]
pub(crate) struct ReceiverInner {
    cookie: ChannelCookie,
    state: Option<ReceiverInnerState>,
}

#[derive(Debug)]
struct ReceiverInnerState {
    client: Handle,
    items: mpsc::UnboundedReceiver<SerializedValue>,
    max_capacity: NonZeroU32,
    cur_capacity: u32,
}

impl ReceiverInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        items: mpsc::UnboundedReceiver<SerializedValue>,
        max_capacity: NonZeroU32,
    ) -> Self {
        Self {
            cookie,
            state: Some(ReceiverInnerState {
                client,
                items,
                max_capacity,
                cur_capacity: max_capacity.get(),
            }),
        }
    }

    fn cookie(&self) -> ChannelCookie {
        self.cookie
    }

    fn client(&self) -> Option<&Handle> {
        self.state.as_ref().map(|s| &s.client)
    }

    async fn close(&mut self) -> Result<(), Error> {
        if let Some(state) = self.state.take() {
            state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Receiver, true)?
                .await
        } else {
            Ok(())
        }
    }

    fn poll_next_item(&mut self, cx: &mut Context) -> Poll<Option<SerializedValue>> {
        let Some(ref mut state) = self.state else {
            return Poll::Ready(None);
        };

        debug_assert!(state.cur_capacity > 0);
        debug_assert!(state.cur_capacity <= state.max_capacity.get());

        let item = match Pin::new(&mut state.items).poll_next(cx) {
            Poll::Ready(Some(item)) => item,
            Poll::Ready(None) => {
                self.state = None;
                return Poll::Ready(None);
            }
            Poll::Pending => return Poll::Pending,
        };

        state.cur_capacity -= 1;
        if state.cur_capacity <= LOW_CAPACITY {
            let diff = state.max_capacity.get() - state.cur_capacity;
            debug_assert!(diff >= 1);

            let _ = state.client.add_channel_capacity(self.cookie, diff);
            state.cur_capacity += diff;
        }

        debug_assert!(state.cur_capacity > 0);
        debug_assert!(state.cur_capacity <= state.max_capacity.get());

        Poll::Ready(Some(item))
    }

    fn is_terminated(&self) -> bool {
        if let Some(ref state) = self.state {
            state.items.is_terminated()
        } else {
            true
        }
    }
}

impl Drop for ReceiverInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            let _ = state
                .client
                .close_channel_end(self.cookie, ChannelEnd::Receiver, true);
        }
    }
}
