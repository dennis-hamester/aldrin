use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CloseChannelEndReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: CloseChannelEndResult,
}

impl CloseChannelEndReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CloseChannelEndReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::CloseChannelEndReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;
        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self {
            serial,
            result: self.result,
        })
    }
}

impl TryFrom<message::CloseChannelEndReply> for CloseChannelEndReply {
    type Error = Error;

    fn try_from(msg: message::CloseChannelEndReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum CloseChannelEndResult {
    Ok,
    InvalidChannel,
    ForeignChannel,
}

impl CloseChannelEndResult {
    pub(crate) fn to_core(self, _ctx: &Context) -> Result<message::CloseChannelEndResult> {
        match self {
            Self::Ok => Ok(message::CloseChannelEndResult::Ok),
            Self::InvalidChannel => Ok(message::CloseChannelEndResult::InvalidChannel),
            Self::ForeignChannel => Ok(message::CloseChannelEndResult::ForeignChannel),
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self == other)
    }
}

impl From<message::CloseChannelEndResult> for CloseChannelEndResult {
    fn from(res: message::CloseChannelEndResult) -> Self {
        match res {
            message::CloseChannelEndResult::Ok => Self::Ok,
            message::CloseChannelEndResult::InvalidChannel => Self::InvalidChannel,
            message::CloseChannelEndResult::ForeignChannel => Self::ForeignChannel,
        }
    }
}
