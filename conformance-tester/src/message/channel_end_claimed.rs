use super::ChannelEndWithCapacity;
use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ChannelEndClaimed {
    pub cookie: UuidRef,

    #[serde(flatten)]
    pub end: ChannelEndWithCapacity,
}

impl ChannelEndClaimed {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::ChannelEndClaimed> {
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::ChannelEndClaimed {
            cookie,
            end: self.end.into(),
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.cookie.matches(&other.cookie, ctx)? && (self.end == other.end);

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self {
            cookie,
            end: self.end,
        })
    }
}

impl TryFrom<message::ChannelEndClaimed> for ChannelEndClaimed {
    type Error = Error;

    fn try_from(msg: message::ChannelEndClaimed) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
            end: msg.end.into(),
        })
    }
}
