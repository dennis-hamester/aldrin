use super::{Receiver, Sender, UnclaimedReceiver, UnclaimedSender};
use crate::{Error, Handle};
use aldrin_core::tags::{self, PrimaryTag, Tag};
use aldrin_core::{
    ChannelCookie, Deserialize, DeserializeError, Deserializer, Serialize, SerializeError,
    Serializer,
};

/// A sender that isn't bound to any client.
///
/// [`UnboundSender`s](Self) are not associated with any client and thus have limited
/// functionality. This type is however the only one, which can be sent to and received from other
/// clients.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnboundSender {
    cookie: ChannelCookie,
}

impl UnboundSender {
    /// Creates a new [`UnboundSender`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self { cookie }
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(self) -> ChannelCookie {
        self.cookie
    }

    /// Casts to a high-level [`UnboundSender`](crate::UnboundSender) by binding an item type `T`.
    pub fn cast<T>(self) -> crate::UnboundSender<T> {
        crate::UnboundSender::new(self.cookie)
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
    /// # let (sender, _) = handle.create_low_level_channel().claim_receiver(1).await?;
    /// # let sender = sender.unbind();
    /// // Assume you got a sender from e.g. the call of some service's function.
    /// // let sender = ...
    ///
    /// // Bind the sender to the local client:
    /// let sender = sender.bind(handle.clone());
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(self, client: Handle) -> UnclaimedSender {
        UnclaimedSender::new(client, self.cookie)
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
    /// # let (sender, _receiver) = handle.create_low_level_channel().claim_receiver(1).await?;
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
    pub async fn claim(self, client: Handle) -> Result<Sender, Error> {
        self.bind(client).claim().await
    }
}

impl From<ChannelCookie> for UnboundSender {
    fn from(cookie: ChannelCookie) -> Self {
        Self::new(cookie)
    }
}

impl PrimaryTag for UnboundSender {
    type Tag = tags::Sender<tags::Value>;
}

impl<T: Tag> Serialize<tags::Sender<T>> for UnboundSender {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(self.cookie)
    }
}

impl<T: Tag> Serialize<tags::Sender<T>> for &UnboundSender {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Sender<T>>(*self)
    }
}

impl<T: Tag> Deserialize<tags::Sender<T>> for UnboundSender {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(Self::new)
    }
}

/// A receiver that isn't bound to any client.
///
/// [`UnboundReceiver`s](Self) are not associated with any client and thus have limited
/// functionality. This type is however the only one, which can be sent to and received from other
/// clients.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnboundReceiver {
    cookie: ChannelCookie,
}

impl UnboundReceiver {
    /// Creates a new [`UnboundReceiver`] from a [`ChannelCookie`].
    pub fn new(cookie: ChannelCookie) -> Self {
        Self { cookie }
    }

    /// Returns the [`ChannelCookie`], which identifies this channel.
    pub fn cookie(self) -> ChannelCookie {
        self.cookie
    }

    /// Casts to a high-level [`UnboundReceiver`](crate::UnboundReceiver) by binding an item type
    /// `T`.
    pub fn cast<T>(self) -> crate::UnboundReceiver<T> {
        crate::UnboundReceiver::new(self.cookie)
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
    /// # let (_, receiver) = handle.create_low_level_channel().claim_sender().await?;
    /// # let receiver = receiver.unbind();
    /// // Assume you got a receiver from e.g. the call of some service's function.
    /// // let receiver = ...
    ///
    /// // Bind the receiver to the local client:
    /// let receiver = receiver.bind(handle.clone());
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(self, client: Handle) -> UnclaimedReceiver {
        UnclaimedReceiver::new(client, self.cookie)
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
    /// # let (sender, receiver) = handle.create_low_level_channel().claim_sender().await?;
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
    /// let item = receiver.next_item::<String>().await?;
    /// assert_eq!(item.as_deref(), Some("Hello :)"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn claim(self, client: Handle, capacity: u32) -> Result<Receiver, Error> {
        self.bind(client).claim(capacity).await
    }
}

impl From<ChannelCookie> for UnboundReceiver {
    fn from(cookie: ChannelCookie) -> Self {
        Self::new(cookie)
    }
}

impl PrimaryTag for UnboundReceiver {
    type Tag = tags::Receiver<tags::Value>;
}

impl<T: Tag> Serialize<tags::Receiver<T>> for UnboundReceiver {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(self.cookie)
    }
}

impl<T: Tag> Serialize<tags::Receiver<T>> for &UnboundReceiver {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Receiver<T>>(*self)
    }
}

impl<T: Tag> Deserialize<tags::Receiver<T>> for UnboundReceiver {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(Self::new)
    }
}
