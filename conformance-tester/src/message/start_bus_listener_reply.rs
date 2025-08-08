use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct StartBusListenerReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: StartBusListenerResult,
}

impl StartBusListenerReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::StartBusListenerReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::StartBusListenerReply { serial, result })
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

impl TryFrom<message::StartBusListenerReply> for StartBusListenerReply {
    type Error = Error;

    fn try_from(msg: message::StartBusListenerReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum StartBusListenerResult {
    Ok,
    InvalidBusListener,
    AlreadyStarted,
}

impl StartBusListenerResult {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::StartBusListenerResult> {
        match self {
            Self::Ok => Ok(message::StartBusListenerResult::Ok),
            Self::InvalidBusListener => Ok(message::StartBusListenerResult::InvalidBusListener),
            Self::AlreadyStarted => Ok(message::StartBusListenerResult::AlreadyStarted),
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

impl From<message::StartBusListenerResult> for StartBusListenerResult {
    fn from(res: message::StartBusListenerResult) -> Self {
        match res {
            message::StartBusListenerResult::Ok => Self::Ok,
            message::StartBusListenerResult::InvalidBusListener => Self::InvalidBusListener,
            message::StartBusListenerResult::AlreadyStarted => Self::AlreadyStarted,
        }
    }
}
