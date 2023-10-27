use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BusListenerCurrentFinished {
    pub cookie: UuidRef,
}

impl BusListenerCurrentFinished {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::BusListenerCurrentFinished> {
        let cookie = self.cookie.get(ctx)?.into();
        Ok(message::BusListenerCurrentFinished { cookie })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.cookie.matches(&other.cookie, ctx)?;
        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.cookie.update_context(&other.cookie, ctx)?;
        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let cookie = self.cookie.apply_context(ctx)?;
        Ok(Self { cookie })
    }
}

impl TryFrom<message::BusListenerCurrentFinished> for BusListenerCurrentFinished {
    type Error = Error;

    fn try_from(msg: message::BusListenerCurrentFinished) -> Result<Self> {
        Ok(Self {
            cookie: msg.cookie.into(),
        })
    }
}
