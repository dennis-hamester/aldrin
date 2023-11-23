use crate::context::Context;
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use aldrin_core::message;
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmitEvent {
    pub service_cookie: UuidRef,
    pub event: u32,

    #[serde(flatten)]
    pub value: Value,
}

impl EmitEvent {
    pub fn to_core(&self, ctx: &Context) -> Result<message::EmitEvent> {
        let service_cookie = self.service_cookie.get(ctx)?.into();

        message::EmitEvent::with_serialize_value(service_cookie, self.event, &self.value)
            .with_context(|| anyhow!("failed to serialize value"))
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
            value: self.value.clone(),
        })
    }
}

impl TryFrom<message::EmitEvent> for EmitEvent {
    type Error = Error;

    fn try_from(msg: message::EmitEvent) -> Result<Self> {
        let value = msg
            .value
            .deserialize()
            .with_context(|| anyhow!("failed to deserialize value `{:?}`", msg.value))?;

        Ok(Self {
            service_cookie: msg.service_cookie.into(),
            event: msg.event,
            value,
        })
    }
}
