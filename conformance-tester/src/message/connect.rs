use crate::context::Context;
use aldrin_core::{message, SerializedValue};
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Connect {
    pub version: u32,
}

impl Connect {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::Connect> {
        let value =
            SerializedValue::serialize(()).with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::Connect {
            version: self.version,
            value,
        })
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self.version == other.version)
    }

    pub(crate) fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::Connect> for Connect {
    type Error = Error;

    fn try_from(msg: message::Connect) -> Result<Self> {
        Ok(Self {
            version: msg.version,
        })
    }
}
