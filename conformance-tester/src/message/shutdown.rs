use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Shutdown;

impl TryFrom<message::Shutdown> for Shutdown {
    type Error = Error;

    fn try_from(_msg: message::Shutdown) -> Result<Self> {
        Ok(Self)
    }
}
