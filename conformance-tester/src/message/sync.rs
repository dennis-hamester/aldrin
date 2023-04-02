use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Sync {
    pub serial: Serial,
}

impl Sync {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::Sync> {
        let serial = self.serial.get(ctx)?;

        Ok(message::Sync { serial })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.serial.matches(&other.serial, ctx)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self { serial })
    }
}

impl TryFrom<message::Sync> for Sync {
    type Error = Error;

    fn try_from(msg: message::Sync) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
        })
    }
}