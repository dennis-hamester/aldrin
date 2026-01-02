use crate::context::Context;
use aldrin_core::SerializedValue;
use aldrin_core::message::{self, ConnectData};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Connect2 {
    pub major_version: u32,
    pub minor_version: u32,
}

impl Connect2 {
    pub(crate) fn to_core(self, _ctx: &Context) -> Result<message::Connect2> {
        let value = SerializedValue::serialize(ConnectData::new())
            .with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::Connect2 {
            major_version: self.major_version,
            minor_version: self.minor_version,
            value,
        })
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        (self.major_version == other.major_version) && (self.minor_version == other.minor_version)
    }

    pub(crate) fn apply_context(self, _ctx: &Context) -> Self {
        self
    }
}

impl TryFrom<message::Connect2> for Connect2 {
    type Error = Error;

    fn try_from(msg: message::Connect2) -> Result<Self> {
        Ok(Self {
            major_version: msg.major_version,
            minor_version: msg.minor_version,
        })
    }
}
