use super::ChannelEnd;
use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateChannel {
    pub serial: Serial,
    pub claim: ChannelEnd,
}

impl CreateChannel {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::CreateChannel> {
        let serial = self.serial.get(ctx)?;

        Ok(message::CreateChannel {
            serial,
            claim: self.claim.into(),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)? && (self.claim == other.claim);

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
            claim: self.claim,
        })
    }
}

impl TryFrom<message::CreateChannel> for CreateChannel {
    type Error = Error;

    fn try_from(msg: message::CreateChannel) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            claim: msg.claim.into(),
        })
    }
}
