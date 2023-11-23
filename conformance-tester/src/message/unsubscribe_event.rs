use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnsubscribeEvent {
    pub service_cookie: UuidRef,
    pub event: u32,
}

impl UnsubscribeEvent {
    pub fn to_core(&self, ctx: &Context) -> Result<message::UnsubscribeEvent> {
        let service_cookie = self.service_cookie.get(ctx)?.into();

        Ok(message::UnsubscribeEvent {
            service_cookie,
            event: self.event,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.service_cookie.matches(&other.service_cookie, ctx)? && (self.event == other.event);

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.service_cookie
            .update_context(&other.service_cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self {
            service_cookie,
            event: self.event,
        })
    }
}

impl TryFrom<message::UnsubscribeEvent> for UnsubscribeEvent {
    type Error = Error;

    fn try_from(msg: message::UnsubscribeEvent) -> Result<Self> {
        Ok(Self {
            service_cookie: msg.service_cookie.into(),
            event: msg.event,
        })
    }
}
