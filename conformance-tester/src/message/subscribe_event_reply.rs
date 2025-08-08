use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SubscribeEventReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: SubscribeEventResult,
}

impl SubscribeEventReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::SubscribeEventReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::SubscribeEventReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;
        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;
        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::SubscribeEventReply> for SubscribeEventReply {
    type Error = Error;

    fn try_from(msg: message::SubscribeEventReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum SubscribeEventResult {
    Ok,
    InvalidService,
}

impl SubscribeEventResult {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::SubscribeEventResult> {
        match self {
            Self::Ok => Ok(message::SubscribeEventResult::Ok),
            Self::InvalidService => Ok(message::SubscribeEventResult::InvalidService),
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self == other)
    }

    pub(crate) fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl From<message::SubscribeEventResult> for SubscribeEventResult {
    fn from(res: message::SubscribeEventResult) -> Self {
        match res {
            message::SubscribeEventResult::Ok => Self::Ok,
            message::SubscribeEventResult::InvalidService => Self::InvalidService,
        }
    }
}
