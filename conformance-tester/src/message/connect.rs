use crate::context::Context;
use crate::value::Value;
use aldrin_core::message;
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Connect {
    pub version: u32,

    #[serde(flatten)]
    pub value: Value,
}

impl Connect {
    pub fn to_core(&self, _ctx: &Context) -> Result<message::Connect> {
        message::Connect::with_serialize_value(self.version, &self.value)
            .with_context(|| anyhow!("failed to serialize value"))
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok((self.version == other.version) && self.value.matches(&other.value))
    }

    pub fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::Connect> for Connect {
    type Error = Error;

    fn try_from(msg: message::Connect) -> Result<Self> {
        let value = msg
            .value
            .deserialize()
            .with_context(|| anyhow!("failed to deserialize value `{:?}`", msg.value))?;

        Ok(Self {
            version: msg.version,
            value,
        })
    }
}
