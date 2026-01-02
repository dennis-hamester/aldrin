use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DestroyServiceReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: DestroyServiceResult,
}

impl DestroyServiceReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::DestroyServiceReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx);

        Ok(message::DestroyServiceReply { serial, result })
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

impl TryFrom<message::DestroyServiceReply> for DestroyServiceReply {
    type Error = Error;

    fn try_from(msg: message::DestroyServiceReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum DestroyServiceResult {
    Ok,
    InvalidService,
    ForeignObject,
}

impl DestroyServiceResult {
    pub(crate) fn to_core(self, _ctx: &Context) -> message::DestroyServiceResult {
        match self {
            Self::Ok => message::DestroyServiceResult::Ok,
            Self::InvalidService => message::DestroyServiceResult::InvalidService,
            Self::ForeignObject => message::DestroyServiceResult::ForeignObject,
        }
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        self == other
    }

    pub(crate) fn apply_context(self, _ctx: &Context) -> Self {
        self
    }
}

impl From<message::DestroyServiceResult> for DestroyServiceResult {
    fn from(res: message::DestroyServiceResult) -> Self {
        match res {
            message::DestroyServiceResult::Ok => Self::Ok,
            message::DestroyServiceResult::InvalidService => Self::InvalidService,
            message::DestroyServiceResult::ForeignObject => Self::ForeignObject,
        }
    }
}
