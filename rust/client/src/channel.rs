use super::Handle;
use crate::error::Error;
use aldrin_proto::{ChannelCookie, ChannelEnd, ConversionError, FromValue, IntoValue, Value};
use futures_channel::{mpsc, oneshot};
use futures_core::stream::{FusedStream, Stream};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A sender that is not bound to any particular client.
///
/// [`UnboundSender`s](Self) are used to transfer senders to some other client, typically by
/// returning them from function calls.
///
/// When [creating a channel](Handle::create_channel_with_claimed_receiver) the resulting
/// [`UnclaimedSender`] can be [unbound](UnclaimedSender::unbind) and converted to a [`Value`] with
/// the [`IntoValue`] trait.
///
/// The other way to obtain an `UnboundSender` reverses the above process by converting [`Value`]
/// back with the [`FromValue`] trait.
///
/// It is worth noting that this type implements [`Copy`] and [`Clone`]. As such (and because it is
/// not bound to any client), it will not destroy the sending end of a channel. This is the main
/// difference from `UnclaimedSender`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnboundSender<T: IntoValue> {
    cookie: ChannelCookie,
    phantom: PhantomData<fn(T)>,
}

impl<T: IntoValue> UnboundSender<T> {
    fn new(cookie: ChannelCookie) -> Self {
        Self {
            cookie,
            phantom: PhantomData,
        }
    }

    /// Binds the sender to a client.
    ///
    /// See also [`claim`](Self::claim) to bind and claim the sender in one step.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{UnclaimedSender, Sender};
    ///
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
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
    /// use aldrin_client::Sender;
    ///
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
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
}

impl<T: IntoValue> IntoValue for UnboundSender<T> {
    fn into_value(self) -> Value {
        Value::Sender(self.cookie)
    }
}

impl<T: IntoValue> FromValue for UnboundSender<T> {
    fn from_value(v: Value) -> Result<Self, ConversionError> {
        match v {
            Value::Sender(cookie) => Ok(Self::new(cookie)),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

/// A sender that hasn't been claimed yet.
///
/// [`UnclaimedSender`s](Self) are similar to [`UnboundSender`s](UnboundSender) in that they
/// identify the sending end of a channel in an unclaimed state. This sender is however bound to a
/// client and can thus be claimed.
#[derive(Debug)]
pub struct UnclaimedSender<T: IntoValue> {
    inner: UnclaimedSenderInner,

    // This is only for the contravariance. We don't care about dropck because the Ts are converted
    // to Values.
    phantom: PhantomData<fn(T)>,
}

impl<T: IntoValue> UnclaimedSender<T> {
    pub(crate) fn new(inner: UnclaimedSenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
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
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Create a channel with an unclaimed sender and a claimed receiver.
    /// let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
    ///
    /// // Unbind the sender so that it can be sent to another client. This will typically happen by
    /// // returning it from a function call.
    /// let sender = sender.unbind();
    /// # Ok(())
    /// # }
    pub fn unbind(self) -> UnboundSender<T> {
        UnboundSender::new(self.inner.unbind())
    }

    /// Destroys the sender without consuming it.
    ///
    /// This destroys the sender such that it cannot be claimed anymore by any client. When the
    /// receiver waits for the channel to become [established](PendingReceiver::established), an
    /// error will be returned.
    ///
    /// After destroying a sender, any further function calls will return [`Error::InvalidChannel`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::Error;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
    ///
    /// // Destroy the sender.
    /// sender.destroy().await?;
    ///
    /// // For the receiver, an error will be returned when waiting for the channel to become
    /// // established.
    /// let err = receiver.established().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
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
    /// - Some other client has destroyed the sender.
    /// - The receiver has been destroyed.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::StreamExt;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // The sender is unclaimed, while the receiver has been claimed automatically.
    /// let (sender, receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
    ///
    /// // Claim the sender.
    /// let mut sender = sender.claim().await?;
    ///
    /// // This will now resolve immediately.
    /// let mut receiver = receiver.established().await?;
    ///
    /// // The channel is now fully established and items can be sent and received.
    /// sender.send(1)?;
    /// sender.send(2)?;
    /// assert_eq!(receiver.next().await, Some(Ok(1)));
    /// assert_eq!(receiver.next().await, Some(Ok(2)));
    /// # Ok(())
    /// # }
    pub async fn claim(self) -> Result<Sender<T>, Error> {
        self.inner.claim().await.map(Sender::new)
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

    fn unbind(mut self) -> ChannelCookie {
        self.client = None;
        self.cookie
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Sender, false)
            .await
    }

    async fn claim(mut self) -> Result<SenderInner, Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client.claim_sender(self.cookie).await
    }
}

impl Drop for UnclaimedSenderInner {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            client.destroy_channel_end_now(self.cookie, ChannelEnd::Sender, false);
        }
    }
}

/// A claimed sender that is waiting for the channel to become established.
///
/// [`PendingSender`s](Self) are used to wait until some client has claimed the receiving end of the
/// channel. This is done with the [`established`](Self::established) function.
#[derive(Debug)]
pub struct PendingSender<T: IntoValue> {
    inner: PendingSenderInner,

    // This is only for the contravariance. We don't care about dropck because the Ts are converted
    // to Values.
    phantom: PhantomData<fn(T)>,
}

impl<T: IntoValue> PendingSender<T> {
    pub(crate) fn new(inner: PendingSenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Destroys the sender without consuming it.
    ///
    /// When destroying a [`PendingSender`], it will no longer be possible to claim the receiver. If
    /// it has already been claimed, then it will receive `None`, indicating that the channel has
    /// been destroyed.
    ///
    /// # Examples
    ///
    /// ## Destroying a sender while the receiver hasn't been claimed yet
    ///
    /// ```
    /// use aldrin_client::Error;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Destroy the sender.
    /// sender.destroy().await?;
    ///
    /// // For the receiver, an error will be returned when trying to claim it.
    /// let err = receiver.claim().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Destroying a sender while the receiver has already been claimed
    ///
    /// ```
    /// use aldrin_client::Error;
    /// use futures::StreamExt;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (mut sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Claim the receiver.
    /// let mut receiver = receiver.claim().await?;
    ///
    /// // Destroy the sender.
    /// sender.destroy().await?;
    ///
    /// // The receiver will receive None.
    /// assert_eq!(receiver.next().await, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
    }

    /// Waits until the channel has been established.
    ///
    /// A channel is established when both ends have been claimed. An error is returned when the
    /// receiver has been destroyed instead of claimed.
    pub async fn established(self) -> Result<Sender<T>, Error> {
        self.inner.established().await.map(Sender::new)
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
    established: oneshot::Receiver<oneshot::Receiver<()>>,
}

impl PendingSenderInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        established: oneshot::Receiver<oneshot::Receiver<()>>,
    ) -> Self {
        Self {
            cookie,
            state: Some(PendingSenderInnerState {
                client,
                established,
            }),
        }
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.state.take().ok_or(Error::InvalidChannel)?.client;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Sender, true)
            .await
    }

    async fn established(mut self) -> Result<SenderInner, Error> {
        let state = self.state.take().ok_or(Error::InvalidChannel)?;
        let client = state.client;

        state
            .established
            .await
            .map(|receiver_destroyed| SenderInner::new(self.cookie, client, receiver_destroyed))
            .map_err(|_| Error::InvalidChannel)
    }
}

impl Drop for PendingSenderInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state
                .client
                .destroy_channel_end_now(self.cookie, ChannelEnd::Sender, true);
        }
    }
}

/// The sending end of an established channel.
///
/// This type of sender is obtained when a channel has been fully established, either by
/// [`PendingSender::established`] or by [`UnclaimedSender::claim`].
#[derive(Debug)]
pub struct Sender<T: IntoValue> {
    inner: SenderInner,

    // This is only for the contravariance. We don't care about dropck because the Ts are converted
    // to Values.
    phantom: PhantomData<fn(T)>,
}

impl<T: IntoValue> Sender<T> {
    pub(crate) fn new(inner: SenderInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Destroys the sender without consuming it.
    ///
    /// The will cause the receiving end to receive [`None`] after all other items have been
    /// received.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::StreamExt;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// let mut receiver = receiver.claim().await?;
    /// let mut sender = sender.established().await?;
    ///
    /// // Send a couple of items and then destroy the sender.
    /// sender.send(1)?;
    /// sender.send(2)?;
    /// sender.send(3)?;
    /// sender.destroy().await?;
    ///
    /// // The receiver will receive all items followed by None.
    /// assert_eq!(receiver.next().await, Some(Ok(1)));
    /// assert_eq!(receiver.next().await, Some(Ok(2)));
    /// assert_eq!(receiver.next().await, Some(Ok(3)));
    /// assert_eq!(receiver.next().await, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
    }

    /// Sends and item on the channel.
    ///
    /// When the receiver is destroyed, then one of the following calls of this function will return
    /// [`Error::InvalidChannel`]. There is no guarantee as to when this will happen. All items sent
    /// after the broker has acknowledged the receiver's destruction will be discarded.
    ///
    /// Note that this function is not `async`. Sending many items in a burst can thus block a task
    /// if this is called in an asynchronous context. This can even block a destruction notification
    /// from the receiver, such that `send` will never indicate an error. It is generally advised to
    /// yield back to the executer regularly.
    pub fn send(&mut self, item: T) -> Result<(), Error> {
        self.inner.send(item.into_value())
    }
}

#[derive(Debug)]
pub(crate) struct SenderInner {
    cookie: ChannelCookie,
    state: Option<SenderInnerState>,
}

#[derive(Debug)]
struct SenderInnerState {
    client: Handle,
    receiver_destroyed: oneshot::Receiver<()>,
}

impl SenderInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        receiver_destroyed: oneshot::Receiver<()>,
    ) -> Self {
        Self {
            cookie,
            state: Some(SenderInnerState {
                client,
                receiver_destroyed,
            }),
        }
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.state.take().ok_or(Error::InvalidChannel)?.client;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Sender, true)
            .await
    }

    fn send(&mut self, item: Value) -> Result<(), Error> {
        let state = self.state.as_mut().ok_or(Error::InvalidChannel)?;

        state.client.send_item(self.cookie, item)?;

        match state.receiver_destroyed.try_recv() {
            Ok(None) => Ok(()),
            Ok(Some(())) | Err(_) => Err(Error::InvalidChannel),
        }
    }
}

impl Drop for SenderInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state
                .client
                .destroy_channel_end_now(self.cookie, ChannelEnd::Sender, true);
        }
    }
}

/// A receiver that is not bound to any particular client.
///
/// [`UnboundReceiver`s](Self) are used to transfer receivers to some other client, typically by
/// returning them from function calls.
///
/// When [creating a channel](Handle::create_channel_with_claimed_sender) the resulting
/// [`UnclaimedReceiver`] can be [unbound](UnclaimedReceiver::unbind) and converted to a [`Value`]
/// with the [`IntoValue`] trait.
///
/// The other way to obtain an `UnboundReceiver` reverses the above process by converting [`Value`]
/// back with the [`FromValue`] trait.
///
/// It is worth noting that this type implements [`Copy`] and [`Clone`]. As such (and because it is
/// not bound to any client), it will not destroy the receiving end of a channel. This is the main
/// difference from `UnclaimedReceiver`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnboundReceiver<T: FromValue> {
    cookie: ChannelCookie,
    phantom: PhantomData<fn() -> T>,
}

impl<T: FromValue> UnboundReceiver<T> {
    fn new(cookie: ChannelCookie) -> Self {
        Self {
            cookie,
            phantom: PhantomData,
        }
    }

    /// Binds the receiver to a client.
    ///
    /// See also [`claim`](Self::claim) to bind and claim the receiver in one step.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{Receiver, UnclaimedReceiver};
    ///
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
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
    /// let receiver: Receiver<u32> = receiver.claim().await?;
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
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::Receiver;
    ///
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume this receiver has been returned from some function call.
    /// // let receiver: UnboundReceiver<u32> = ...
    ///
    /// // Bind it to the local client and claim it, so that it can immediately be used. The
    /// // explicit type here is given only for the sake of the example.
    /// let receiver: Receiver<u32> = receiver.claim(handle.clone()).await?;
    /// # Ok(())
    /// # }
    pub async fn claim(self, client: Handle) -> Result<Receiver<T>, Error> {
        self.bind(client).claim().await
    }
}

impl<T: FromValue> IntoValue for UnboundReceiver<T> {
    fn into_value(self) -> Value {
        Value::Receiver(self.cookie)
    }
}

impl<T: FromValue> FromValue for UnboundReceiver<T> {
    fn from_value(v: Value) -> Result<Self, ConversionError> {
        match v {
            Value::Receiver(cookie) => Ok(Self::new(cookie)),
            _ => Err(ConversionError(Some(v))),
        }
    }
}

/// A receiver that hasn't been claimed yet.
///
/// [`UnclaimedReceiver`s](Self) are similar to [`UnboundReceiver`s](UnboundReceiver) in that they
/// identify the receiving end of a channel in an unclaimed state. This receiver is however bound to
/// a client and can thus be claimed.
#[derive(Debug)]
pub struct UnclaimedReceiver<T: FromValue> {
    inner: UnclaimedReceiverInner,

    // This is only for the covariance. We don't care about dropck because the Ts are produced on
    // the fly from Values.
    phantom: PhantomData<fn() -> T>,
}

impl<T: FromValue> UnclaimedReceiver<T> {
    pub(crate) fn new(inner: UnclaimedReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
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
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
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

    /// Destroys the receiver without consuming it.
    ///
    /// This destroys the receiver such that it cannot be claimed anymore by any client. When the
    /// sender waits for the channel to become [established](PendingSender::established), an error
    /// will be returned.
    ///
    /// After destroying a receiver, any further function calls will return
    /// [`Error::InvalidChannel`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::Error;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Destroy the receiver.
    /// receiver.destroy().await?;
    ///
    /// // For the sender, an error will be returned when waiting for the channel to become
    /// // established.
    /// let err = sender.established().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
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
    /// - Some other client has destroyed the receiver.
    /// - The sender has been destroyed.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::StreamExt;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // The receiver is unclaimed, while the sender has been claimed automatically.
    /// let (sender, receiver) = handle.create_channel_with_claimed_sender::<u32>().await?;
    ///
    /// // Claim the receiver.
    /// let mut receiver = receiver.claim().await?;
    ///
    /// // This will now resolve immediately.
    /// let mut sender = sender.established().await?;
    ///
    /// // The channel is now fully established and items can be sent and received.
    /// sender.send(1)?;
    /// sender.send(2)?;
    /// assert_eq!(receiver.next().await, Some(Ok(1)));
    /// assert_eq!(receiver.next().await, Some(Ok(2)));
    /// # Ok(())
    /// # }
    pub async fn claim(self) -> Result<Receiver<T>, Error> {
        self.inner.claim().await.map(Receiver::new)
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

    fn unbind(mut self) -> ChannelCookie {
        self.client = None;
        self.cookie
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Receiver, false)
            .await
    }

    async fn claim(mut self) -> Result<ReceiverInner, Error> {
        let client = self.client.take().ok_or(Error::InvalidChannel)?;
        client.claim_receiver(self.cookie).await
    }
}

impl Drop for UnclaimedReceiverInner {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            client.destroy_channel_end_now(self.cookie, ChannelEnd::Receiver, false);
        }
    }
}

/// A claimed receiver that is waiting for the channel to become established.
///
/// [`PendingReceiver`s](Self) are used to wait until some client has claimed the sending end of the
/// channel. This is done with the [`established`](Self::established) function.
#[derive(Debug)]
pub struct PendingReceiver<T: FromValue> {
    inner: PendingReceiverInner,

    // This is only for the covariance. We don't care about dropck because the Ts are produced on
    // the fly from Values.
    phantom: PhantomData<fn() -> T>,
}

impl<T: FromValue> PendingReceiver<T> {
    pub(crate) fn new(inner: PendingReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Destroys the receiver without consuming it.
    ///
    /// When destroying a [`PendingReceiver`], it will no longer be possible to claim the sender.
    ///
    /// When the sender has already been claimed, the situation is a little bit more
    /// complicated. The sender is notified asynchronously about the receiver's destruction. It
    /// will, sooner or later, receive an error when sending an item.
    ///
    /// # Examples
    ///
    /// ## Destroying a receiver while the sender hasn't been claimed yet
    ///
    /// ```
    /// use aldrin_client::Error;
    /// # use aldrin_test::tokio_based::TestBroker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// let (sender, mut receiver) = handle.create_channel_with_claimed_receiver::<u32>().await?;
    ///
    /// // Destroy the receiver.
    /// receiver.destroy().await?;
    ///
    /// // For the sender, an error will be returned when trying to claim it.
    /// let err = sender.claim().await.unwrap_err();
    /// assert_eq!(err, Error::InvalidChannel);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
    }

    /// Waits until the channel has been established.
    ///
    /// A channel is established when both ends have been claimed. An error is returned when the
    /// sender has been destroyed instead of claimed.
    pub async fn established(self) -> Result<Receiver<T>, Error> {
        self.inner.established().await.map(Receiver::new)
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
    established: oneshot::Receiver<mpsc::UnboundedReceiver<Value>>,
}

impl PendingReceiverInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        established: oneshot::Receiver<mpsc::UnboundedReceiver<Value>>,
    ) -> Self {
        Self {
            cookie,
            state: Some(PendingReceiverInnerState {
                client,
                established,
            }),
        }
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.state.take().ok_or(Error::InvalidChannel)?.client;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Receiver, true)
            .await
    }

    async fn established(mut self) -> Result<ReceiverInner, Error> {
        let state = self.state.take().ok_or(Error::InvalidChannel)?;
        let client = state.client;

        state
            .established
            .await
            .map(|items| ReceiverInner::new(self.cookie, client, items))
            .map_err(|_| Error::InvalidChannel)
    }
}

impl Drop for PendingReceiverInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state
                .client
                .destroy_channel_end_now(self.cookie, ChannelEnd::Receiver, true);
        }
    }
}

/// The receiving end of an established channel.
///
/// This type of receiver is obtained when a channel has been fully established, either by
/// [`PendingReceiver::established`] or by [`UnclaimedReceiver::claim`].
#[derive(Debug)]
pub struct Receiver<T: FromValue> {
    inner: ReceiverInner,

    // This is only for the covariance. We don't care about dropck because the Ts are produced on
    // the fly from Values.
    phantom: PhantomData<fn() -> T>,
}

impl<T: FromValue> Receiver<T> {
    pub(crate) fn new(inner: ReceiverInner) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Destroys the receiver without consuming it.
    ///
    /// The sender will be notified asynchronously and cause [`Sender::send`] to return
    /// [`Error::InvalidChannel`].
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.inner.destroy().await
    }
}

impl<T: FromValue> Stream for Receiver<T> {
    type Item = Result<T, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(
                item.convert().map_err(|e| Error::InvalidItemReceived(e.0)),
            )),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: FromValue> FusedStream for Receiver<T> {
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
    items: mpsc::UnboundedReceiver<Value>,
}

impl ReceiverInner {
    pub(crate) fn new(
        cookie: ChannelCookie,
        client: Handle,
        items: mpsc::UnboundedReceiver<Value>,
    ) -> Self {
        Self {
            cookie,
            state: Some(ReceiverInnerState { client, items }),
        }
    }

    async fn destroy(&mut self) -> Result<(), Error> {
        let client = self.state.take().ok_or(Error::InvalidChannel)?.client;
        client
            .destroy_channel_end(self.cookie, ChannelEnd::Receiver, true)
            .await
    }
}

impl Drop for ReceiverInner {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state
                .client
                .destroy_channel_end_now(self.cookie, ChannelEnd::Receiver, true);
        }
    }
}

impl Stream for ReceiverInner {
    type Item = Value;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Value>> {
        match self.state.as_mut() {
            Some(state) => Pin::new(&mut state.items).poll_next(cx),
            None => Poll::Ready(None),
        }
    }
}

impl FusedStream for ReceiverInner {
    fn is_terminated(&self) -> bool {
        match self.state.as_ref() {
            Some(state) => state.items.is_terminated(),
            None => true,
        }
    }
}
