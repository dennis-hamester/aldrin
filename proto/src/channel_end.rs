use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Sending or receiving end of a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum ChannelEnd {
    /// Sending end of a channel.
    Sender = 0,

    /// Receiving end of a channel.
    Receiver = 1,
}

impl ChannelEnd {
    /// Returns the other end of the channel.
    ///
    /// This function maps [`Sender`](Self::Sender) to [`Receiver`](Self::Receiver) and vice versa.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::message::ChannelEnd;
    /// assert_eq!(ChannelEnd::Sender.other(), ChannelEnd::Receiver);
    /// assert_eq!(ChannelEnd::Receiver.other(), ChannelEnd::Sender);
    /// ```
    pub fn other(self) -> Self {
        match self {
            Self::Sender => Self::Receiver,
            Self::Receiver => Self::Sender,
        }
    }
}

impl From<ChannelEndWithCapacity> for ChannelEnd {
    fn from(value: ChannelEndWithCapacity) -> Self {
        match value {
            ChannelEndWithCapacity::Sender => Self::Sender,
            ChannelEndWithCapacity::Receiver(_) => Self::Receiver,
        }
    }
}

/// Sending or receiving end and capacity of a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum ChannelEndWithCapacity {
    /// Sending end of a channel.
    Sender,

    /// Receiving end of a channel and capacity.
    Receiver(u32),
}
