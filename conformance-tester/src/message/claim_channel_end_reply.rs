use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClaimChannelEndReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: ClaimChannelEndResult,
}

impl ClaimChannelEndReply {
    pub fn to_core(&self, ctx: &Context) -> Result<message::ClaimChannelEndReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::ClaimChannelEndReply { serial, result })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self {
            serial,
            result: self.result,
        })
    }
}

impl TryFrom<message::ClaimChannelEndReply> for ClaimChannelEndReply {
    type Error = Error;

    fn try_from(msg: message::ClaimChannelEndReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum ClaimChannelEndResult {
    SenderClaimed { capacity: u32 },
    ReceiverClaimed,
    InvalidChannel,
    AlreadyClaimed,
}

impl ClaimChannelEndResult {
    pub fn to_core(self, _ctx: &Context) -> Result<message::ClaimChannelEndResult> {
        match self {
            Self::SenderClaimed { capacity } => {
                Ok(message::ClaimChannelEndResult::SenderClaimed(capacity))
            }

            Self::ReceiverClaimed => Ok(message::ClaimChannelEndResult::ReceiverClaimed),
            Self::InvalidChannel => Ok(message::ClaimChannelEndResult::InvalidChannel),
            Self::AlreadyClaimed => Ok(message::ClaimChannelEndResult::AlreadyClaimed),
        }
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self == other)
    }
}

impl From<message::ClaimChannelEndResult> for ClaimChannelEndResult {
    fn from(res: message::ClaimChannelEndResult) -> Self {
        match res {
            message::ClaimChannelEndResult::SenderClaimed(capacity) => {
                Self::SenderClaimed { capacity }
            }

            message::ClaimChannelEndResult::ReceiverClaimed => Self::ReceiverClaimed,
            message::ClaimChannelEndResult::InvalidChannel => Self::InvalidChannel,
            message::ClaimChannelEndResult::AlreadyClaimed => Self::AlreadyClaimed,
        }
    }
}
