use crate::context::Context;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnsubscribeObjects;

impl UnsubscribeObjects {
    pub fn to_proto(&self, _ctx: &Context) -> Result<message::UnsubscribeObjects> {
        Ok(message::UnsubscribeObjects)
    }

    pub fn matches(&self, _other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(true)
    }

    pub fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::UnsubscribeObjects> for UnsubscribeObjects {
    type Error = Error;

    fn try_from(_msg: message::UnsubscribeObjects) -> Result<Self> {
        Ok(Self)
    }
}
