use crate::bus_listener::BusListenerScope;
use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StartBusListener {
    pub serial: Serial,
    pub cookie: UuidRef,
    pub scope: BusListenerScope,
}

impl StartBusListener {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::StartBusListener> {
        let serial = self.serial.get(ctx)?;
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::StartBusListener {
            serial,
            cookie,
            scope: self.scope.into(),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.cookie.matches(&other.cookie, ctx)?
            && (self.scope == other.scope);

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self {
            serial,
            cookie,
            scope: self.scope,
        })
    }
}

impl TryFrom<message::StartBusListener> for StartBusListener {
    type Error = Error;

    fn try_from(msg: message::StartBusListener) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            cookie: msg.cookie.into(),
            scope: msg.scope.into(),
        })
    }
}
