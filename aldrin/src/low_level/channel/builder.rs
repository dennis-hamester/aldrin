use super::{PendingReceiver, PendingSender, UnclaimedReceiver, UnclaimedSender};
use crate::{Error, Handle};

/// Builder type for creating channels.
///
/// Create a [`ChannelBuilder`] either with [`Handle::create_low_level_channel`] or
/// [`ChannelBuilder::new`].
///
/// The [`ChannelBuilder`] is used to specify which of the channels ends will be claimed by the
/// local client.
///
/// Note that this type is [`Copy`] and is thus not consumed by any of its methods.
#[derive(Debug, Copy, Clone)]
pub struct ChannelBuilder<'a> {
    client: &'a Handle,
}

impl<'a> ChannelBuilder<'a> {
    /// Creates a new [`ChannelBuilder`].
    ///
    /// Alternatively, [`Handle::create_low_level_channel`] can be more convenient to use, because
    /// it usually avoids the need to import [`ChannelBuilder`].
    ///
    /// ```
    /// # use aldrin::low_level::ChannelBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker = aldrin_test::tokio::TestBroker::new();
    /// # let handle = broker.add_client().await;
    /// // Option 1
    /// let builder = ChannelBuilder::new(&handle);
    ///
    /// // Option 2
    /// let builder = handle.create_low_level_channel();
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(client: &'a Handle) -> Self {
        Self { client }
    }

    /// Returns a [`Handle`] to the associated client.
    pub fn client(self) -> &'a Handle {
        self.client
    }

    /// Casts to a high-level [`ChannelBuilder`](crate::ChannelBuilder) by binding an item type
    /// `T`.
    pub fn cast<T>(self) -> crate::ChannelBuilder<'a, T> {
        crate::ChannelBuilder::new(self.client)
    }

    /// Creates a new channel and claims the sender.
    pub async fn claim_sender(self) -> Result<(PendingSender, UnclaimedReceiver), Error> {
        self.client.create_claimed_sender().await
    }

    /// Creates a new channel and claims the receiver.
    ///
    /// A capacity of 0 will be treated as if 1 was specified instead.
    pub async fn claim_receiver(
        self,
        capacity: u32,
    ) -> Result<(UnclaimedSender, PendingReceiver), Error> {
        self.client.create_claimed_receiver(capacity).await
    }
}
