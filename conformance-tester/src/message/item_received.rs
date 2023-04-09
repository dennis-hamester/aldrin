use crate::context::Context;
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use aldrin_proto::message;
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ItemReceived {
    pub cookie: UuidRef,

    #[serde(flatten)]
    pub value: Value,
}

impl ItemReceived {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::ItemReceived> {
        let cookie = self.cookie.get(ctx)?.into();

        message::ItemReceived::with_serialize_value(cookie, &self.value)
            .with_context(|| anyhow!("failed to serialize value"))
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.cookie.matches(&other.cookie, ctx)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self {
            cookie,
            value: self.value.clone(),
        })
    }
}

impl TryFrom<message::ItemReceived> for ItemReceived {
    type Error = Error;

    fn try_from(msg: message::ItemReceived) -> Result<Self> {
        let value = msg
            .value
            .deserialize()
            .with_context(|| anyhow!("failed to deserialize value `{:?}`", msg.value))?;

        Ok(Self {
            cookie: msg.cookie.into(),
            value,
        })
    }
}
