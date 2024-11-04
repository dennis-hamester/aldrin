use super::{Receiver, Sender, UnclaimedReceiver, UnclaimedSender};
#[cfg(feature = "introspection")]
use crate::core::introspection::{
    BuiltInType, DynIntrospectable, Introspectable, Layout, LexicalId,
};
use crate::core::{
    AsSerializeArg, ChannelCookie, Deserialize, DeserializeError, Deserializer, Serialize,
    SerializeError, Serializer,
};
use crate::error::Error;
use crate::handle::Handle;
use crate::low_level;
use std::fmt;
use std::marker::PhantomData;

/// A sender that isn't bound to any client.
///
/// [`UnboundSender`s](Self) are not associated with any client and thus have limited
/// functionality. This type is however the only one, which can be sent to and received from other
/// clients.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
pub struct UnboundSender<T: ?Sized> {
    inner: low_level::UnboundSender,
    phantom: PhantomData<fn(T)>,
}

impl<T: ?Sized> UnboundSender<T> {
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
    pub fn cast<U: ?Sized>(self) -> UnboundSender<U> {
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

impl<T: ?Sized> From<ChannelCookie> for UnboundSender<T> {
    fn from(cookie: ChannelCookie) -> Self {
        Self::new(cookie)
    }
}

impl<T: ?Sized> fmt::Debug for UnboundSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnboundSender")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: ?Sized> Clone for UnboundSender<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for UnboundSender<T> {}

impl<T: ?Sized> Serialize for UnboundSender<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        self.inner.serialize(serializer)
    }
}

impl<T: ?Sized> Deserialize for UnboundSender<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        low_level::UnboundSender::deserialize(deserializer).map(low_level::UnboundSender::cast)
    }
}

impl<T: ?Sized> AsSerializeArg for UnboundSender<T> {
    type SerializeArg<'a>
        = Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
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

    fn add_references(references: &mut Vec<DynIntrospectable>) {
        references.push(DynIntrospectable::new::<T>());
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

impl<T> Serialize for UnboundReceiver<T> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        self.inner.serialize(serializer)
    }
}

impl<T> Deserialize for UnboundReceiver<T> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        low_level::UnboundReceiver::deserialize(deserializer).map(low_level::UnboundReceiver::cast)
    }
}

impl<T> AsSerializeArg for UnboundReceiver<T> {
    type SerializeArg<'a>
        = Self
    where
        Self: 'a;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        *self
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

    fn add_references(references: &mut Vec<DynIntrospectable>) {
        references.push(DynIntrospectable::new::<T>());
    }
}
