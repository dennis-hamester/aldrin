use super::{Receiver, Sender, UnclaimedReceiver, UnclaimedSender};
use crate::{low_level, Error, Handle};
#[cfg(feature = "introspection")]
use aldrin_core::introspection::{ir, Introspectable, LexicalId, References};
use aldrin_core::tags::{self, PrimaryTag, Tag};
use aldrin_core::{
    ChannelCookie, Deserialize, DeserializeError, Deserializer, Serialize, SerializeError,
    Serializer,
};
use std::fmt;
use std::marker::PhantomData;

/// A sender that isn't bound to any client.
///
/// [`UnboundSender`s](Self) are not associated with any client and thus have limited
/// functionality. This type is however the only one, which can be sent to and received from other
/// clients.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
pub struct UnboundSender<T> {
    inner: low_level::UnboundSender,
    phantom: PhantomData<fn(T)>,
}

impl<T> UnboundSender<T> {
    /// Creates a new [`UnboundSender`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self {
            inner: low_level::UnboundSender::new(cookie),
            phantom: PhantomData,
        }
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts the item type to a different type `U`.
    pub fn cast<U>(self) -> UnboundSender<U> {
        UnboundSender::new(self.cookie())
    }

    /// Converts the sender into a low-level [`UnboundSender`](low_level::UnboundSender).
    pub fn as_low_level(self) -> low_level::UnboundSender {
        self.inner
    }

    /// Binds the sender to a client and creates an [`UnclaimedSender`].
    ///
    /// After acquiring an [`UnboundSender`], it must first be bound to a client and then claimed in
    /// order to fully establish the channel. This method performs the first step of that process
    /// and yields an [`UnclaimedSender`].
    ///
    /// There is also [`claim`](Self::claim), which performs both steps at once.
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, _) = handle.create_channel::<()>().claim_receiver(1).await?;
    /// # let sender = sender.unbind();
    /// // Assume you got a sender from e.g. the call of some service's function.
    /// // let sender = ...
    ///
    /// // Bind the sender to the local client:
    /// let sender = sender.bind(handle.clone());
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(self, client: Handle) -> UnclaimedSender<T> {
        low_level::UnclaimedSender::new(client, self.cookie()).cast()
    }

    /// Binds the sender to a client and claims it.
    ///
    /// This is a shorthand for calling [`bind`](Self::bind) followed by
    /// [`UnclaimedSender::claim`]. If successful, this fully establishes the channel and returns a
    /// [`Sender`].
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, _receiver) = handle.create_channel::<String>().claim_receiver(1).await?;
    /// # let sender = sender.unbind();
    /// // Assume you got a sender from e.g. the call of some service's function.
    /// // let sender = ...
    ///
    /// // Bind and claim the sender:
    /// let mut sender = sender.claim(handle.clone()).await?;
    ///
    /// // The channel is now established and items can be sent:
    /// sender.send_item("Hello :)").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn claim(self, client: Handle) -> Result<Sender<T>, Error> {
        self.bind(client).claim().await
    }
}

impl<T> From<ChannelCookie> for UnboundSender<T> {
    fn from(cookie: ChannelCookie) -> Self {
        Self::new(cookie)
    }
}

impl<T> fmt::Debug for UnboundSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnboundSender")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Clone for UnboundSender<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnboundSender<T> {}

impl<T: PrimaryTag> PrimaryTag for UnboundSender<T> {
    type Tag = tags::Sender<T::Tag>;
}

impl<T: Tag, U: Serialize<T>> Serialize<tags::Sender<T>> for UnboundSender<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Sender<T>>(self.inner)
    }
}

impl<T: Tag, U: Serialize<T>> Serialize<tags::Sender<T>> for &UnboundSender<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl<T: Tag, U: Serialize<T>> Deserialize<tags::Sender<T>> for UnboundSender<U> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<tags::Sender<T>, _>()
            .map(low_level::UnboundSender::cast)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for UnboundSender<T> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Sender(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::sender(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}

/// A receiver that isn't bound to any client.
///
/// [`UnboundReceiver`s](Self) are not associated with any client and thus have limited
/// functionality. This type is however the only one, which can be sent to and received from other
/// clients.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
pub struct UnboundReceiver<T> {
    inner: low_level::UnboundReceiver,
    phantom: PhantomData<fn() -> T>,
}

impl<T> UnboundReceiver<T> {
    /// Creates a new [`UnboundReceiver`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self {
            inner: low_level::UnboundReceiver::new(cookie),
            phantom: PhantomData,
        }
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(self) -> ChannelCookie {
        self.inner.cookie()
    }

    /// Casts the item type to a different type `U`.
    pub fn cast<U>(self) -> UnboundReceiver<U> {
        UnboundReceiver::new(self.cookie())
    }

    /// Converts the receiver into a low-level [`UnboundReceiver`](low_level::UnboundReceiver).
    pub fn as_low_level(self) -> low_level::UnboundReceiver {
        self.inner
    }

    /// Binds the receiver to a client and creates an [`UnclaimedReceiver`].
    ///
    /// After acquiring an [`UnboundReceiver`], it must first be bound to a client and then claimed
    /// in order to fully establish the channel. This method performs the first step of that process
    /// and yields an [`UnclaimedReceiver`].
    ///
    /// There is also [`claim`](Self::claim), which performs both steps at once.
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (_, receiver) = handle.create_channel::<()>().claim_sender().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume you got a receiver from e.g. the call of some service's function.
    /// // let receiver = ...
    ///
    /// // Bind the receiver to the local client:
    /// let receiver = receiver.bind(handle.clone());
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(self, client: Handle) -> UnclaimedReceiver<T> {
        low_level::UnclaimedReceiver::new(client, self.cookie()).cast()
    }

    /// Binds the receiver to a client and claims it.
    ///
    /// This is a shorthand for calling [`bind`](Self::bind) followed by
    /// [`UnclaimedReceiver::claim`]. If successful, this fully establishes the channel and returns
    /// a [`Receiver`].
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// # let (sender, receiver) = handle.create_channel::<String>().claim_sender().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume you got a receiver from e.g. the call of some service's function.
    /// // let receiver = ...
    ///
    /// // Bind and claim the receiver:
    /// let mut receiver = receiver.claim(handle.clone(), 16).await?;
    /// # let mut sender = sender.establish().await?;
    /// # sender.send_item("Hello :)").await?;
    ///
    /// // The channel is now established and items can be received:
    /// let item = receiver.next_item().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn claim(self, client: Handle, capacity: u32) -> Result<Receiver<T>, Error> {
        self.bind(client).claim(capacity).await
    }
}

impl<T> From<ChannelCookie> for UnboundReceiver<T> {
    fn from(cookie: ChannelCookie) -> Self {
        Self::new(cookie)
    }
}

impl<T> fmt::Debug for UnboundReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnclaimedReceiver")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Clone for UnboundReceiver<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnboundReceiver<T> {}

impl<T: PrimaryTag> PrimaryTag for UnboundReceiver<T> {
    type Tag = tags::Receiver<T::Tag>;
}

impl<T: Tag, U: Deserialize<T>> Serialize<tags::Receiver<T>> for UnboundReceiver<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Receiver<T>>(self.inner)
    }
}

impl<T: Tag, U: Deserialize<T>> Serialize<tags::Receiver<T>> for &UnboundReceiver<U> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl<T: Tag, U: Deserialize<T>> Deserialize<tags::Receiver<T>> for UnboundReceiver<U> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<tags::Receiver<T>, _>()
            .map(low_level::UnboundReceiver::cast)
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable> Introspectable for UnboundReceiver<T> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Receiver(T::lexical_id()).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::receiver(T::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
    }
}
