use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DestroyBusListenerReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: DestroyBusListenerResult,
}

impl DestroyBusListenerReply {
    pub fn to_core(&self, ctx: &Context) -> Result<message::DestroyBusListenerReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::DestroyBusListenerReply { serial, result })
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

impl TryFrom<message::DestroyBusListenerReply> for DestroyBusListenerReply {
    type Error = Error;

    fn try_from(msg: message::DestroyBusListenerReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum DestroyBusListenerResult {
    Ok,
    InvalidBusListener,
}

impl DestroyBusListenerResult {
    pub fn to_core(&self, _ctx: &Context) -> Result<message::DestroyBusListenerResult> {
        match self {
            Self::Ok => Ok(message::DestroyBusListenerResult::Ok),
            Self::InvalidBusListener => Ok(message::DestroyBusListenerResult::InvalidBusListener),
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

impl From<message::DestroyBusListenerResult> for DestroyBusListenerResult {
    fn from(res: message::DestroyBusListenerResult) -> Self {
        match res {
            message::DestroyBusListenerResult::Ok => Self::Ok,
            message::DestroyBusListenerResult::InvalidBusListener => Self::InvalidBusListener,
        }
    }
}
