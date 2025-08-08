use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CreateObjectReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: CreateObjectResult,
}

impl CreateObjectReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CreateObjectReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::CreateObjectReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::CreateObjectReply> for CreateObjectReply {
    type Error = Error;

    fn try_from(msg: message::CreateObjectReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum CreateObjectResult {
    Ok { cookie: UuidRef },
    DuplicateObject,
}

impl CreateObjectResult {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CreateObjectResult> {
        match self {
            Self::Ok { cookie } => {
                let cookie = cookie.get(ctx)?.into();
                Ok(message::CreateObjectResult::Ok(cookie))
            }

            Self::DuplicateObject => Ok(message::CreateObjectResult::DuplicateObject),
        }
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { cookie: c1 }, Self::Ok { cookie: c2 }) => c1.matches(c2, ctx),
            (Self::DuplicateObject, Self::DuplicateObject) => Ok(true),
            _ => Ok(false),
        }
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::Ok { cookie: c1 }, Self::Ok { cookie: c2 }) => c1.update_context(c2, ctx),
            (Self::DuplicateObject, Self::DuplicateObject) => Ok(()),
            _ => unreachable!(),
        }
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Ok { cookie } => {
                let cookie = cookie.apply_context(ctx)?;
                Ok(Self::Ok { cookie })
            }

            Self::DuplicateObject => Ok(Self::DuplicateObject),
        }
    }
}

impl From<message::CreateObjectResult> for CreateObjectResult {
    fn from(res: message::CreateObjectResult) -> Self {
        match res {
            message::CreateObjectResult::Ok(cookie) => Self::Ok {
                cookie: cookie.into(),
            },

            message::CreateObjectResult::DuplicateObject => Self::DuplicateObject,
        }
    }
}
