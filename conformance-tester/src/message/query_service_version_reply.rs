use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QueryServiceVersionReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: QueryServiceVersionResult,
}

impl QueryServiceVersionReply {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::QueryServiceVersionReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_proto(ctx)?;

        Ok(message::QueryServiceVersionReply { serial, result })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self {
            serial,
            result: self.result.clone(),
        })
    }
}

impl TryFrom<message::QueryServiceVersionReply> for QueryServiceVersionReply {
    type Error = Error;

    fn try_from(msg: message::QueryServiceVersionReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum QueryServiceVersionResult {
    Ok { version: u32 },
    InvalidService,
}

impl QueryServiceVersionResult {
    pub fn to_proto(&self, _ctx: &Context) -> Result<message::QueryServiceVersionResult> {
        match self {
            Self::Ok { version } => Ok(message::QueryServiceVersionResult::Ok(*version)),
            Self::InvalidService => Ok(message::QueryServiceVersionResult::InvalidService),
        }
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { version: v1 }, Self::Ok { version: v2 }) => Ok(v1 == v2),
            (Self::InvalidService, Self::InvalidService) => Ok(true),
            _ => Ok(false),
        }
    }
}

impl From<message::QueryServiceVersionResult> for QueryServiceVersionResult {
    fn from(res: message::QueryServiceVersionResult) -> Self {
        match res {
            message::QueryServiceVersionResult::Ok(version) => Self::Ok { version },
            message::QueryServiceVersionResult::InvalidService => Self::InvalidService,
        }
    }
}
