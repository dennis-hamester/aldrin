use crate::context::Context;
use aldrin_core::message::{self, ConnectData};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Connect2 {
    pub major_version: u32,
    pub minor_version: u32,
}

impl Connect2 {
    pub fn to_core(&self, _ctx: &Context) -> Result<message::Connect2> {
        Ok(message::Connect2::with_serialize_data(
            self.major_version,
            self.minor_version,
            &ConnectData::new(),
        )
        .unwrap())
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok((self.major_version == other.major_version)
            && (self.minor_version == other.minor_version))
    }

    pub fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::Connect2> for Connect2 {
    type Error = Error;

    fn try_from(msg: message::Connect2) -> Result<Self> {
        Ok(Self {
            major_version: msg.major_version,
            minor_version: msg.minor_version,
        })
    }
}
