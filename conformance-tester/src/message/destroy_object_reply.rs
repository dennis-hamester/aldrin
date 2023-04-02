use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DestroyObjectReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: DestroyObjectResult,
}

impl DestroyObjectReply {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::DestroyObjectReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_proto(ctx)?;

        Ok(message::DestroyObjectReply { serial, result })
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

impl TryFrom<message::DestroyObjectReply> for DestroyObjectReply {
    type Error = Error;

    fn try_from(msg: message::DestroyObjectReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum DestroyObjectResult {
    Ok,
    InvalidObject,
    ForeignObject,
}

impl DestroyObjectResult {
    pub fn to_proto(&self, _ctx: &Context) -> Result<message::DestroyObjectResult> {
        match self {
            Self::Ok => Ok(message::DestroyObjectResult::Ok),
            Self::InvalidObject => Ok(message::DestroyObjectResult::InvalidObject),
            Self::ForeignObject => Ok(message::DestroyObjectResult::ForeignObject),
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

impl From<message::DestroyObjectResult> for DestroyObjectResult {
    fn from(res: message::DestroyObjectResult) -> Self {
        match res {
            message::DestroyObjectResult::Ok => Self::Ok,
            message::DestroyObjectResult::InvalidObject => Self::InvalidObject,
            message::DestroyObjectResult::ForeignObject => Self::ForeignObject,
        }
    }
}
