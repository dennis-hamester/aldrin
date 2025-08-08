use super::ChannelEndWithCapacity;
use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ClaimChannelEnd {
    pub serial: Serial,
    pub cookie: UuidRef,

    #[serde(flatten)]
    pub end: ChannelEndWithCapacity,
}

impl ClaimChannelEnd {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::ClaimChannelEnd> {
        let serial = self.serial.get(ctx)?;
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::ClaimChannelEnd {
            serial,
            cookie,
            end: self.end.into(),
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.cookie.matches(&other.cookie, ctx)?
            && (self.end == other.end);

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self {
            serial,
            cookie,
            end: self.end,
        })
    }
}

impl TryFrom<message::ClaimChannelEnd> for ClaimChannelEnd {
    type Error = Error;

    fn try_from(msg: message::ClaimChannelEnd) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            cookie: msg.cookie.into(),
            end: msg.end.into(),
        })
    }
}
