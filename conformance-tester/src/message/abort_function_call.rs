use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct AbortFunctionCall {
    pub serial: Serial,
}

impl AbortFunctionCall {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::AbortFunctionCall> {
        let serial = self.serial.get(ctx)?;

        Ok(message::AbortFunctionCall { serial })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.serial.matches(&other.serial, ctx)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self { serial })
    }
}

impl TryFrom<message::AbortFunctionCall> for AbortFunctionCall {
    type Error = Error;

    fn try_from(msg: message::AbortFunctionCall) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
        })
    }
}
