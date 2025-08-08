use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ServiceDestroyed {
    pub service_cookie: UuidRef,
}

impl ServiceDestroyed {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::ServiceDestroyed> {
        let service_cookie = self.service_cookie.get(ctx)?.into();

        Ok(message::ServiceDestroyed { service_cookie })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        self.service_cookie.matches(&other.service_cookie, ctx)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.service_cookie
            .update_context(&other.service_cookie, ctx)
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self { service_cookie })
    }
}

impl TryFrom<message::ServiceDestroyed> for ServiceDestroyed {
    type Error = Error;

    fn try_from(msg: message::ServiceDestroyed) -> Result<Self> {
        Ok(Self {
            service_cookie: msg.service_cookie.into(),
        })
    }
}
