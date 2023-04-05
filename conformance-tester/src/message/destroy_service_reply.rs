use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DestroyServiceReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: DestroyServiceResult,
}

impl DestroyServiceReply {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::DestroyServiceReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_proto(ctx)?;

        Ok(message::DestroyServiceReply { serial, result })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;
        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;
        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum DestroyServiceResult {
    Ok,
    InvalidService,
    ForeignObject,
}

impl DestroyServiceResult {
    pub fn to_proto(&self, _ctx: &Context) -> Result<message::DestroyServiceResult> {
        match self {
            Self::Ok => Ok(message::DestroyServiceResult::Ok),
            Self::InvalidService => Ok(message::DestroyServiceResult::InvalidService),
            Self::ForeignObject => Ok(message::DestroyServiceResult::ForeignObject),
        }
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self == other)
    }

    pub fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
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
