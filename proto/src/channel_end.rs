use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
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

impl Serialize for ChannelEnd {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Sender => serializer.serialize_enum(0, &()),
            Self::Receiver => serializer.serialize_enum(1, &()),
        }
    }
}

impl Deserialize for ChannelEnd {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.variant() {
            0 => deserializer.deserialize().map(|()| Self::Sender),
            1 => deserializer.deserialize().map(|()| Self::Receiver),
            _ => Err(DeserializeError::InvalidSerialization),
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
