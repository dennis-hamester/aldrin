use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateBusListener {
    pub serial: Serial,
}

impl CreateBusListener {
    pub fn to_core(&self, ctx: &Context) -> Result<message::CreateBusListener> {
        let serial = self.serial.get(ctx)?;

        Ok(message::CreateBusListener { serial })
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

impl TryFrom<message::CreateBusListener> for CreateBusListener {
    type Error = Error;

    fn try_from(msg: message::CreateBusListener) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
        })
    }
}
