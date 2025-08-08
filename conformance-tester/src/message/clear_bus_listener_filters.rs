use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ClearBusListenerFilters {
    pub cookie: UuidRef,
}

impl ClearBusListenerFilters {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::ClearBusListenerFilters> {
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::ClearBusListenerFilters { cookie })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.cookie.matches(&other.cookie, ctx)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self { cookie })
    }
}

impl TryFrom<message::ClearBusListenerFilters> for ClearBusListenerFilters {
    type Error = Error;

    fn try_from(msg: message::ClearBusListenerFilters) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
        })
    }
}
