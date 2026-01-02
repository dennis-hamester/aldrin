use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct UnsubscribeAllEventsReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: UnsubscribeAllEventsResult,
}

impl UnsubscribeAllEventsReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::UnsubscribeAllEventsReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx);

        Ok(message::UnsubscribeAllEventsReply { serial, result })
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

impl TryFrom<message::UnsubscribeAllEventsReply> for UnsubscribeAllEventsReply {
    type Error = Error;

    fn try_from(msg: message::UnsubscribeAllEventsReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum UnsubscribeAllEventsResult {
    Ok,
    InvalidService,
    NotSupported,
}

impl UnsubscribeAllEventsResult {
    pub(crate) fn to_core(self, _ctx: &Context) -> message::UnsubscribeAllEventsResult {
        match self {
            Self::Ok => message::UnsubscribeAllEventsResult::Ok,
            Self::InvalidService => message::UnsubscribeAllEventsResult::InvalidService,
            Self::NotSupported => message::UnsubscribeAllEventsResult::NotSupported,
        }
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        self == other
    }

    pub(crate) fn apply_context(self, _ctx: &Context) -> Self {
        self
    }
}

impl From<message::UnsubscribeAllEventsResult> for UnsubscribeAllEventsResult {
    fn from(res: message::UnsubscribeAllEventsResult) -> Self {
        match res {
            message::UnsubscribeAllEventsResult::Ok => Self::Ok,
            message::UnsubscribeAllEventsResult::InvalidService => Self::InvalidService,
            message::UnsubscribeAllEventsResult::NotSupported => Self::NotSupported,
        }
    }
}
