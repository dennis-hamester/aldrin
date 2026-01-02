use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SubscribeServiceReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: SubscribeServiceResult,
}

impl SubscribeServiceReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::SubscribeServiceReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx);

        Ok(message::SubscribeServiceReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(other.result, ctx);
        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx);

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::SubscribeServiceReply> for SubscribeServiceReply {
    type Error = Error;

    fn try_from(msg: message::SubscribeServiceReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum SubscribeServiceResult {
    Ok,
    InvalidService,
}

impl SubscribeServiceResult {
    pub(crate) fn to_core(self, _ctx: &Context) -> message::SubscribeServiceResult {
        match self {
            Self::Ok => message::SubscribeServiceResult::Ok,
            Self::InvalidService => message::SubscribeServiceResult::InvalidService,
        }
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        self == other
    }

    pub(crate) fn apply_context(self, _ctx: &Context) -> Self {
        self
    }
}

impl From<message::SubscribeServiceResult> for SubscribeServiceResult {
    fn from(res: message::SubscribeServiceResult) -> Self {
        match res {
            message::SubscribeServiceResult::Ok => Self::Ok,
            message::SubscribeServiceResult::InvalidService => Self::InvalidService,
        }
    }
}
