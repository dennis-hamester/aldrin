use crate::context::Context;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Connect {
    pub version: u32,
}

impl Connect {
    pub(crate) fn to_core(self, _ctx: &Context) -> Result<message::Connect> {
        let value =
            SerializedValue::serialize(()).with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::Connect {
            version: self.version,
            value,
        })
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        self.version == other.version
    }

    pub(crate) fn apply_context(self, _ctx: &Context) -> Self {
        self
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
