use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct QueryServiceVersionReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: QueryServiceVersionResult,
}

impl QueryServiceVersionReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::QueryServiceVersionReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx);

        Ok(message::QueryServiceVersionReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(other.result, ctx);

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;

        Ok(Self {
            serial,
            result: self.result,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum QueryServiceVersionResult {
    Ok { version: u32 },
    InvalidService,
}

impl QueryServiceVersionResult {
    pub(crate) fn to_core(self, _ctx: &Context) -> message::QueryServiceVersionResult {
        match self {
            Self::Ok { version } => message::QueryServiceVersionResult::Ok(version),
            Self::InvalidService => message::QueryServiceVersionResult::InvalidService,
        }
    }

    pub(crate) fn matches(self, other: Self, _ctx: &Context) -> bool {
        match (self, other) {
            (Self::Ok { version: v1 }, Self::Ok { version: v2 }) => v1 == v2,
            (Self::InvalidService, Self::InvalidService) => true,
            _ => false,
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
