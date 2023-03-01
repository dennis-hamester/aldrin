use crate::conn_id::ConnectionId;
use aldrin_proto::message::{ChannelEnd, ClaimChannelEndResult, CloseChannelEndResult};
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

    pub fn claim_sender(
        &mut self,
        conn_id: &ConnectionId,
    ) -> Result<(&ConnectionId, u32), ClaimChannelEndResult> {
        match self.sender {
            ChannelEndState::Unclaimed => {}
            ChannelEndState::Claimed { .. } => return Err(ClaimChannelEndResult::AlreadyClaimed),
            ChannelEndState::Closed => return Err(ClaimChannelEndResult::InvalidChannel),
        }

        let ChannelEndState::Claimed { owner: ref receiver, capacity } = self.receiver else {
            // The channel is closed before.
            unreachable!();
        };

        self.sender = ChannelEndState::Claimed {
            owner: conn_id.clone(),
            capacity,
        };

        Ok((receiver, capacity))
    }

    pub fn claim_receiver(
        &mut self,
        conn_id: &ConnectionId,
        capacity: u32,
    ) -> Result<&ConnectionId, ClaimChannelEndResult> {
        match self.receiver {
            ChannelEndState::Unclaimed => {}
            ChannelEndState::Claimed { .. } => return Err(ClaimChannelEndResult::AlreadyClaimed),
            ChannelEndState::Closed => return Err(ClaimChannelEndResult::InvalidChannel),
        }

        let ChannelEndState::Claimed {
            owner: ref sender,
            capacity: ref mut sender_capacity,
        } = self.sender else {
            // The channel is closed before.
            unreachable!();
        };

        self.receiver = ChannelEndState::Claimed {
            owner: conn_id.clone(),
            capacity,
        };

        *sender_capacity = capacity;

        Ok(sender)
    }

    pub fn send_item(
        &mut self,
        conn_id: &ConnectionId,
    ) -> Result<(&ConnectionId, Option<u32>), SendItemError> {
        let ChannelEndState::Claimed {
            owner: ref sender,
            capacity: ref mut sender_capacity,
        } = self.sender else {
            return Err(SendItemError::InvalidSender);
        };

        if sender != conn_id {
            return Err(SendItemError::InvalidSender);
        }

        let (receiver, receiver_capacity) = match self.receiver {
            ChannelEndState::Claimed {
                owner: ref receiver,
                capacity: ref mut receiver_capacity,
                ..
            } => (receiver, receiver_capacity),
            ChannelEndState::Unclaimed => return Err(SendItemError::ReceiverUnclaimed),
            ChannelEndState::Closed => return Err(SendItemError::ReceiverClosed),
        };

        if *receiver_capacity == 0 {
            return Err(SendItemError::CapacityExhausted);
        }

        debug_assert!(*sender_capacity > 0);
        *sender_capacity -= 1;
        *receiver_capacity -= 1;

        let add_capacity =
            if (*sender_capacity <= LOW_CAPACITY) && (*receiver_capacity > *sender_capacity) {
                let diff = *receiver_capacity - *sender_capacity;
                *sender_capacity = *receiver_capacity;
                Some(diff)
            } else {
                None
            };

        Ok((receiver, add_capacity))
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
            debug_assert!(*receiver_capacity > *sender_capacity);
            let diff = *receiver_capacity - *sender_capacity;
            *sender_capacity = *receiver_capacity;
            Some((sender, diff))
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
    CapacityExhausted,
}

#[derive(Debug)]
enum ChannelEndState {
    Unclaimed,
    Claimed { owner: ConnectionId, capacity: u32 },
    Closed,
}
