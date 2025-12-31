use super::{PendingReceiver, PendingSender, UnclaimedReceiver, UnclaimedSender};
use crate::{Error, Handle, low_level};
use std::fmt;
use std::marker::PhantomData;

/// Builder type for creating channels.
///
/// Create a [`ChannelBuilder`] either with [`Handle::create_channel`] or [`ChannelBuilder::new`].
///
/// The [`ChannelBuilder`] is used to specify which of the channels ends will be claimed by the
/// local client.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
pub struct ChannelBuilder<'a, T> {
    inner: low_level::ChannelBuilder<'a>,
    phantom: PhantomData<fn(T) -> T>,
}

impl<'a, T> ChannelBuilder<'a, T> {
    /// Creates a new [`ChannelBuilder`].
    ///
    /// Alternatively, [`Handle::create_channel`] can be more convenient to use, because it usually
    /// avoids the need to import [`ChannelBuilder`].
    ///
    /// ```
    /// # use aldrin::ChannelBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Option 1
    /// let builder = ChannelBuilder::<String>::new(&handle);
    ///
    /// // Option 2
    /// let builder = handle.create_channel::<String>();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// It's not usually necessary to specify the item's type explicitly, because it can be deferred
    /// by the compiler in many cases.
    pub fn new(client: &'a Handle) -> Self {
        Self {
            inner: low_level::ChannelBuilder::new(client),
            phantom: PhantomData,
        }
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(self) -> &'a Handle {
        self.inner.client()
    }

    /// Casts the item type to a different type `U`.
    pub fn cast<U>(self) -> ChannelBuilder<'a, U> {
        ChannelBuilder::new(self.client())
    }

    /// Creates a new channel and claims the sender.
    pub async fn claim_sender(self) -> Result<(PendingSender<T>, UnclaimedReceiver<T>), Error> {
        let (sender, receiver) = self.inner.claim_sender().await?;
        Ok((sender.cast(), receiver.cast()))
    }

    /// Creates a new channel and claims the receiver.
    ///
    /// A capacity of 0 will be treated as if 1 was specified instead.
    pub async fn claim_receiver(
        self,
        capacity: u32,
    ) -> Result<(UnclaimedSender<T>, PendingReceiver<T>), Error> {
        let (sender, receiver) = self.inner.claim_receiver(capacity).await?;
        Ok((sender.cast(), receiver.cast()))
    }
}

impl<T> fmt::Debug for ChannelBuilder<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChannelBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Clone for ChannelBuilder<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ChannelBuilder<'_, T> {}
