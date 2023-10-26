use super::bus_listener_filter::BusListenerFilter;
use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddBusListenerFilter {
    pub cookie: UuidRef,

    #[serde(flatten)]
    pub filter: BusListenerFilter,
}

impl AddBusListenerFilter {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::AddBusListenerFilter> {
        let cookie = self.cookie.get(ctx)?.into();
        let filter = self.filter.to_proto(ctx)?;

        Ok(message::AddBusListenerFilter { cookie, filter })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.cookie.matches(&other.cookie, ctx)? && self.filter.matches(&other.filter, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;
        self.filter.update_context(&other.filter, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;
        let filter = self.filter.apply_context(ctx)?;

        Ok(Self { cookie, filter })
    }
}

impl TryFrom<message::AddBusListenerFilter> for AddBusListenerFilter {
    type Error = Error;

    fn try_from(msg: message::AddBusListenerFilter) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
            filter: msg.filter.into(),
        })
    }
}
