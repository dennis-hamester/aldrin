use crate::conn_id::ConnectionId;
use aldrin_proto::message::{ChannelEnd, ClaimChannelEndResult, DestroyChannelEndResult};
use std::mem;

#[derive(Debug)]
pub(crate) struct Channel {
    sender: ChannelEndState,
    receiver: ChannelEndState,
}

impl Channel {
    pub fn with_claimed_sender(conn_id: ConnectionId) -> Self {
        Self {
            sender: ChannelEndState::Claimed(conn_id),
            receiver: ChannelEndState::Unclaimed,
        }
    }

    pub fn with_claimed_receiver(conn_id: ConnectionId) -> Self {
        Self {
            sender: ChannelEndState::Unclaimed,
            receiver: ChannelEndState::Claimed(conn_id),
        }
    }

    pub fn check_destroy(
        &self,
        conn_id: &ConnectionId,
        end: ChannelEnd,
    ) -> (DestroyChannelEndResult, bool) {
        let state = match end {
            ChannelEnd::Sender => &self.sender,
            ChannelEnd::Receiver => &self.receiver,
        };

        match state {
            ChannelEndState::Unclaimed => (DestroyChannelEndResult::Ok, false),
            ChannelEndState::Claimed(owner) if *owner == *conn_id => {
                (DestroyChannelEndResult::Ok, true)
            }
            ChannelEndState::Claimed(_) => (DestroyChannelEndResult::ForeignChannel, true),
            ChannelEndState::Destroyed => (DestroyChannelEndResult::InvalidChannel, false),
        }
    }

    pub fn destroy(&mut self, end: ChannelEnd) -> Option<&ConnectionId> {
        let (owner, other) = match end {
            ChannelEnd::Sender => (&mut self.sender, &self.receiver),
            ChannelEnd::Receiver => (&mut self.receiver, &self.sender),
        };

        // The channel end always gets destroyed.
        let owner = mem::replace(owner, ChannelEndState::Destroyed);

        // Decide whether to destroy the channel (both ends) completely, and whether to notify the
        // owner of the other end.
        match (owner, other) {
            (ChannelEndState::Claimed(_), ChannelEndState::Unclaimed)
            | (ChannelEndState::Claimed(_), ChannelEndState::Destroyed) => None,

            (ChannelEndState::Unclaimed, ChannelEndState::Claimed(other))
            | (ChannelEndState::Claimed(_), ChannelEndState::Claimed(other)) => Some(other),

            // In all of these cases, it's illegal to call destroy.
            (ChannelEndState::Unclaimed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Unclaimed, ChannelEndState::Destroyed)
            | (ChannelEndState::Destroyed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Destroyed, ChannelEndState::Claimed(_))
            | (ChannelEndState::Destroyed, ChannelEndState::Destroyed) => unreachable!(),
        }
    }

    pub fn claim(
        &mut self,
        conn_id: &ConnectionId,
        end: ChannelEnd,
    ) -> Result<&ConnectionId, ClaimChannelEndResult> {
        let (owner, other) = match end {
            ChannelEnd::Sender => (&mut self.sender, &self.receiver),
            ChannelEnd::Receiver => (&mut self.receiver, &self.sender),
        };

        match (owner, other) {
            (owner @ ChannelEndState::Unclaimed, ChannelEndState::Claimed(other)) => {
                *owner = ChannelEndState::Claimed(conn_id.clone());
                Ok(other)
            }

            (ChannelEndState::Claimed(_), ChannelEndState::Unclaimed)
            | (ChannelEndState::Claimed(_), ChannelEndState::Claimed(_))
            | (ChannelEndState::Claimed(_), ChannelEndState::Destroyed) => {
                Err(ClaimChannelEndResult::AlreadyClaimed)
            }

            (ChannelEndState::Destroyed, ChannelEndState::Claimed(_)) => {
                Err(ClaimChannelEndResult::InvalidChannel)
            }

            // The whole channel is destroyed before any of these cases can happen.
            (ChannelEndState::Unclaimed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Unclaimed, ChannelEndState::Destroyed)
            | (ChannelEndState::Destroyed, ChannelEndState::Unclaimed)
            | (ChannelEndState::Destroyed, ChannelEndState::Destroyed) => unreachable!(),
        }
    }

    pub fn check_send(&self, conn_id: &ConnectionId) -> (bool, Option<&ConnectionId>) {
        match &self.sender {
            ChannelEndState::Claimed(sender) if *sender == *conn_id => {
                if let ChannelEndState::Claimed(receiver) = &self.receiver {
                    (true, Some(receiver))
                } else {
                    (true, None)
                }
            }

            _ => (false, None),
        }
    }
}

#[derive(Debug)]
enum ChannelEndState {
    Unclaimed,
    Claimed(ConnectionId),
    Destroyed,
}
