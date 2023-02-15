use crate::conn_id::ConnectionId;
use aldrin_proto::message::{
    ChannelEnd, ChannelEndWithCapacity, ClaimChannelEndResult, CloseChannelEndResult,
};
use std::mem;

const LOW_CAPACITY: u32 = 4;

#[derive(Debug)]
pub(crate) struct Channel {
    sender: ChannelEndState,
    receiver: ChannelEndState,
}

impl Channel {
    pub fn with_claimed_sender(owner: ConnectionId) -> Self {
        Self {
            sender: ChannelEndState::Claimed { owner, capacity: 0 },
            receiver: ChannelEndState::Unclaimed,
        }
    }

    pub fn with_claimed_receiver(owner: ConnectionId, capacity: u32) -> Self {
        Self {
            sender: ChannelEndState::Unclaimed,
            receiver: ChannelEndState::Claimed { owner, capacity },
        }
    }

    pub fn check_close(
        &self,
        conn_id: &ConnectionId,
        end: ChannelEnd,
    ) -> (CloseChannelEndResult, bool) {
        let state = match end {
            ChannelEnd::Sender => &self.sender,
            ChannelEnd::Receiver => &self.receiver,
        };

        match state {
            ChannelEndState::Unclaimed => (CloseChannelEndResult::Ok, false),
            ChannelEndState::Claimed { owner, .. } if *owner == *conn_id => {
                (CloseChannelEndResult::Ok, true)
            }
            ChannelEndState::Claimed { .. } => (CloseChannelEndResult::ForeignChannel, true),
            ChannelEndState::Closed => (CloseChannelEndResult::InvalidChannel, false),
        }
    }

    pub fn close(&mut self, end: ChannelEnd) -> Option<&ConnectionId> {
        let (owner, other) = match end {
            ChannelEnd::Sender => (&mut self.sender, &self.receiver),
            ChannelEnd::Receiver => (&mut self.receiver, &self.sender),
        };

        // The channel end always gets closed.
        let owner = mem::replace(owner, ChannelEndState::Closed);

        // Decide whether to close the channel (both ends) completely, and whether to notify the
        // owner of the other end.
        match (owner, other) {
            (ChannelEndState::Claimed { .. }, ChannelEndState::Unclaimed)
            | (ChannelEndState::Claimed { .. }, ChannelEndState::Closed) => None,

            (ChannelEndState::Unclaimed, ChannelEndState::Claimed { owner: other, .. })
            | (ChannelEndState::Claimed { .. }, ChannelEndState::Claimed { owner: other, .. }) => {
                Some(other)
            }

            // In all of these cases, it's illegal to call close.
            (ChannelEndState::Unclaimed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Unclaimed, ChannelEndState::Closed)
            | (ChannelEndState::Closed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Closed, ChannelEndState::Claimed { .. })
            | (ChannelEndState::Closed, ChannelEndState::Closed) => unreachable!(),
        }
    }

    pub fn claim(
        &mut self,
        conn_id: &ConnectionId,
        end: ChannelEndWithCapacity,
    ) -> Result<&ConnectionId, ClaimChannelEndResult> {
        let (owner, other) = match end {
            ChannelEndWithCapacity::Sender => (&mut self.sender, &self.receiver),
            ChannelEndWithCapacity::Receiver(_) => (&mut self.receiver, &self.sender),
        };

        match (owner, other) {
            (
                owner @ ChannelEndState::Unclaimed,
                ChannelEndState::Claimed {
                    owner: other,
                    capacity,
                },
            ) => {
                *owner = ChannelEndState::Claimed {
                    owner: conn_id.clone(),
                    capacity: *capacity,
                };
                Ok(other)
            }

            (ChannelEndState::Claimed { .. }, ChannelEndState::Unclaimed)
            | (ChannelEndState::Claimed { .. }, ChannelEndState::Claimed { .. })
            | (ChannelEndState::Claimed { .. }, ChannelEndState::Closed) => {
                Err(ClaimChannelEndResult::AlreadyClaimed)
            }

            (ChannelEndState::Closed, ChannelEndState::Claimed { .. }) => {
                Err(ClaimChannelEndResult::InvalidChannel)
            }

            // The whole channel is closed before any of these cases can happen.
            (ChannelEndState::Unclaimed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Unclaimed, ChannelEndState::Closed)
            | (ChannelEndState::Closed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Closed, ChannelEndState::Closed) => unreachable!(),
        }
    }

    pub fn send_item(&self, conn_id: &ConnectionId) -> Result<&ConnectionId, SendItemError> {
        let ChannelEndState::Claimed { owner: ref sender, .. } = self.sender else {
            return Err(SendItemError::InvalidSender);
        };

        if sender != conn_id {
            return Err(SendItemError::InvalidSender);
        }

        match self.receiver {
            ChannelEndState::Claimed {
                owner: ref receiver,
                ..
            } => Ok(receiver),
            ChannelEndState::Unclaimed => Err(SendItemError::ReceiverUnclaimed),
            ChannelEndState::Closed => Err(SendItemError::ReceiverClosed),
        }
    }

    pub fn add_capacity(
        &mut self,
        conn_id: &ConnectionId,
        capacity: u32,
    ) -> Option<(&ConnectionId, u32)> {
        if capacity == 0 {
            return None;
        }

        let ChannelEndState::Claimed {
            owner: ref receiver,
            capacity: ref mut receiver_capacity,
        } = self.receiver else {
            return None;
        };

        if receiver != conn_id {
            return None;
        }

        *receiver_capacity += capacity;

        let ChannelEndState::Claimed {
            owner: ref sender,
            capacity: ref mut sender_capacity,
        } = self.sender else {
            return None;
        };

        if *sender_capacity <= LOW_CAPACITY {
            *sender_capacity = *receiver_capacity;
            Some((sender, *sender_capacity))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SendItemError {
    InvalidSender,
    ReceiverUnclaimed,
    ReceiverClosed,
}

#[derive(Debug)]
enum ChannelEndState {
    Unclaimed,
    Claimed { owner: ConnectionId, capacity: u32 },
    Closed,
}
