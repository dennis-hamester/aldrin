use crate::context::Context;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Shutdown;

impl Shutdown {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::Shutdown> {
        Ok(message::Shutdown)
    }

    pub(crate) fn matches(&self, _other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(true)
    }

    pub(crate) fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::Shutdown> for Shutdown {
    type Error = Error;

    fn try_from(_msg: message::Shutdown) -> Result<Self> {
        Ok(Self)
    }
}
