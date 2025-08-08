use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct AddChannelCapacity {
    pub cookie: UuidRef,
    pub capacity: u32,
}

impl AddChannelCapacity {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::AddChannelCapacity> {
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::AddChannelCapacity {
            cookie,
            capacity: self.capacity,
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.cookie.matches(&other.cookie, ctx)? && (self.capacity == other.capacity);

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
            capacity: self.capacity,
        })
    }
}

impl TryFrom<message::AddChannelCapacity> for AddChannelCapacity {
    type Error = Error;

    fn try_from(msg: message::AddChannelCapacity) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
            capacity: msg.capacity,
        })
    }
}
