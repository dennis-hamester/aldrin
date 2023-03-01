use super::ChannelEnd;
use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ChannelEndClosed {
    pub cookie: UuidRef,
    pub end: ChannelEnd,
}

impl ChannelEndClosed {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::ChannelEndClosed> {
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::ChannelEndClosed {
            cookie,
            end: self.end.into(),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.cookie.matches(&other.cookie, ctx)? && (self.end == other.end);

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self {
            cookie,
            end: self.end,
        })
    }
}

impl TryFrom<message::ChannelEndClosed> for ChannelEndClosed {
    type Error = Error;

    fn try_from(msg: message::ChannelEndClosed) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
            end: msg.end.into(),
        })
    }
}