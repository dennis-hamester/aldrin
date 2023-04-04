use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateServiceReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: CreateServiceResult,
}

impl CreateServiceReply {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::CreateServiceReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_proto(ctx)?;

        Ok(message::CreateServiceReply { serial, result })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;
        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::CreateServiceReply> for CreateServiceReply {
    type Error = Error;

    fn try_from(msg: message::CreateServiceReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum CreateServiceResult {
    Ok { cookie: UuidRef },
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

impl CreateServiceResult {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::CreateServiceResult> {
        match self {
            Self::Ok { cookie } => {
                let cookie = cookie.get(ctx)?.into();
                Ok(message::CreateServiceResult::Ok(cookie))
            }

            Self::DuplicateService => Ok(message::CreateServiceResult::DuplicateService),
            Self::InvalidObject => Ok(message::CreateServiceResult::InvalidObject),
            Self::ForeignObject => Ok(message::CreateServiceResult::ForeignObject),
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { cookie: c1 }, Self::Ok { cookie: c2 }) => c1.matches(c2, ctx),
            (Self::DuplicateService, Self::DuplicateService)
            | (Self::InvalidObject, Self::InvalidObject)
            | (Self::ForeignObject, Self::ForeignObject) => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::Ok { cookie: c1 }, Self::Ok { cookie: c2 }) => c1.update_context(c2, ctx),
            (Self::DuplicateService, Self::DuplicateService)
            | (Self::InvalidObject, Self::InvalidObject)
            | (Self::ForeignObject, Self::ForeignObject) => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Ok { cookie } => {
                let cookie = cookie.apply_context(ctx)?;
                Ok(Self::Ok { cookie })
            }

            Self::DuplicateService => Ok(Self::DuplicateService),
            Self::InvalidObject => Ok(Self::InvalidObject),
            Self::ForeignObject => Ok(Self::ForeignObject),
        }
    }
}

impl From<message::CreateServiceResult> for CreateServiceResult {
    fn from(res: message::CreateServiceResult) -> Self {
        match res {
            message::CreateServiceResult::Ok(cookie) => Self::Ok {
                cookie: cookie.into(),
            },

            message::CreateServiceResult::DuplicateService => Self::DuplicateService,
            message::CreateServiceResult::InvalidObject => Self::InvalidObject,
            message::CreateServiceResult::ForeignObject => Self::ForeignObject,
        }
    }
}
