use super::ChannelEndWithCapacity;
use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateChannel {
    pub serial: Serial,

    #[serde(flatten)]
    pub end: ChannelEndWithCapacity,
}

impl CreateChannel {
    pub fn to_core(&self, ctx: &Context) -> Result<message::CreateChannel> {
        let serial = self.serial.get(ctx)?;

        Ok(message::CreateChannel {
            serial,
            end: self.end.into(),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)? && (self.end == other.end);

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self {
            serial,
            end: self.end,
        })
    }
}

impl TryFrom<message::CreateChannel> for CreateChannel {
    type Error = Error;

    fn try_from(msg: message::CreateChannel) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            end: msg.end.into(),
        })
    }
}
