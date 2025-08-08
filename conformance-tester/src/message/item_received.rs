use crate::context::Context;
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use aldrin_core::{message, SerializedValue};
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ItemReceived {
    pub cookie: UuidRef,

    #[serde(flatten)]
    pub value: Value,
}

impl ItemReceived {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::ItemReceived> {
        let cookie = self.cookie.get(ctx)?.into();

        let value = SerializedValue::serialize(&self.value)
            .with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::ItemReceived { cookie, value })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.cookie.matches(&other.cookie, ctx)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
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
