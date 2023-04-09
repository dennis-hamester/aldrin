use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QueryObjectReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: QueryObjectResult,
}

impl QueryObjectReply {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::QueryObjectReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_proto(ctx)?;

        Ok(message::QueryObjectReply { serial, result })
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

impl TryFrom<message::QueryObjectReply> for QueryObjectReply {
    type Error = Error;

    fn try_from(msg: message::QueryObjectReply) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum QueryObjectResult {
    Cookie { cookie: UuidRef },
    Service { uuid: UuidRef, cookie: UuidRef },
    Done,
    InvalidObject,
}

impl QueryObjectResult {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::QueryObjectResult> {
        match self {
            Self::Cookie { cookie } => {
                let cookie = cookie.get(ctx)?.into();
                Ok(message::QueryObjectResult::Cookie(cookie))
            }

            Self::Service { uuid, cookie } => {
                let uuid = uuid.get(ctx)?.into();
                let cookie = cookie.get(ctx)?.into();
                Ok(message::QueryObjectResult::Service { uuid, cookie })
            }

            Self::Done => Ok(message::QueryObjectResult::Done),
            Self::InvalidObject => Ok(message::QueryObjectResult::InvalidObject),
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Cookie { cookie: c1 }, Self::Cookie { cookie: c2 }) => c1.matches(c2, ctx),

            (
                Self::Service {
                    uuid: u1,
                    cookie: c1,
                },
                Self::Service {
                    uuid: u2,
                    cookie: c2,
                },
            ) => {
                let res = u1.matches(u2, ctx)? && c1.matches(c2, ctx)?;
                Ok(res)
            }

            (Self::Done, Self::Done) | (Self::InvalidObject, Self::InvalidObject) => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::Cookie { cookie: c1 }, Self::Cookie { cookie: c2 }) => {
                c1.update_context(c2, ctx)
            }

            (
                Self::Service {
                    uuid: u1,
                    cookie: c1,
                },
                Self::Service {
                    uuid: u2,
                    cookie: c2,
                },
            ) => {
                u1.update_context(u2, ctx)?;
                c1.update_context(c2, ctx)?;
                Ok(())
            }

            (Self::Done, Self::Done) | (Self::InvalidObject, Self::InvalidObject) => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Cookie { cookie } => {
                let cookie = cookie.apply_context(ctx)?;
                Ok(Self::Cookie { cookie })
            }

            Self::Service { uuid, cookie } => {
                let uuid = uuid.apply_context(ctx)?;
                let cookie = cookie.apply_context(ctx)?;
                Ok(Self::Service { uuid, cookie })
            }

            Self::Done => Ok(Self::Done),
            Self::InvalidObject => Ok(Self::InvalidObject),
        }
    }
}

impl From<message::QueryObjectResult> for QueryObjectResult {
    fn from(res: message::QueryObjectResult) -> Self {
        match res {
            message::QueryObjectResult::Cookie(cookie) => Self::Cookie {
                cookie: cookie.into(),
            },

            message::QueryObjectResult::Service { uuid, cookie } => Self::Service {
                uuid: uuid.into(),
                cookie: cookie.into(),
            },

            message::QueryObjectResult::Done => Self::Done,
            message::QueryObjectResult::InvalidObject => Self::InvalidObject,
        }
    }
}
