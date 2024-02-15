use crate::context::Context;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Connect {
    pub version: u32,
}

impl Connect {
    pub fn to_core(&self, _ctx: &Context) -> Result<message::Connect> {
        Ok(message::Connect::with_serialize_value(self.version, &()).unwrap())
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self.version == other.version)
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
        Ok(Self {
            version: msg.version,
        })
    }
}
