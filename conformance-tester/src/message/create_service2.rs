use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::{message, SerializedValue};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateService2 {
    pub serial: Serial,
    pub object_cookie: UuidRef,
    pub uuid: UuidRef,
    pub info: ServiceInfo,
}

impl CreateService2 {
    pub fn to_core(&self, ctx: &Context) -> Result<message::CreateService2> {
        let serial = self.serial.get(ctx)?;
        let object_cookie = self.object_cookie.get(ctx)?.into();
        let uuid = self.uuid.get(ctx)?.into();
        let value = self.info.serialize(ctx)?;

        Ok(message::CreateService2 {
            serial,
            object_cookie,
            uuid,
            value,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.object_cookie.matches(&other.object_cookie, ctx)?
            && self.uuid.matches(&other.uuid, ctx)?
            && self.info.matches(&other.info, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.object_cookie
            .update_context(&other.object_cookie, ctx)?;
        self.uuid.update_context(&other.uuid, ctx)?;
        self.info.update_context(&other.info, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let object_cookie = self.object_cookie.apply_context(ctx)?;
        let uuid = self.uuid.apply_context(ctx)?;
        let info = self.info.apply_context(ctx)?;

        Ok(Self {
            serial,
            object_cookie,
            uuid,
            info,
        })
    }
}

impl TryFrom<message::CreateService2> for CreateService2 {
    type Error = Error;

    fn try_from(msg: message::CreateService2) -> Result<Self> {
        let info = match msg.deserialize_info() {
            Ok(info) => ServiceInfo::Valid(info.into()),
            Err(_) => ServiceInfo::Invalid,
        };

        Ok(Self {
            serial: msg.serial.into(),
            object_cookie: msg.object_cookie.into(),
            uuid: msg.uuid.into(),
            info,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceInfo {
    Valid(super::ServiceInfo),
    Invalid,
}

impl ServiceInfo {
    fn serialize(&self, ctx: &Context) -> Result<SerializedValue> {
        match self {
            Self::Valid(info) => {
                let info = info.to_core(ctx)?;
                Ok(SerializedValue::serialize(&info).unwrap())
            }

            Self::Invalid => Ok(SerializedValue::serialize(&()).unwrap()),
        }
    }

    fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Valid(info), Self::Valid(other)) => info.matches(other, ctx),
            (Self::Invalid, Self::Invalid) => Ok(true),
            _ => Ok(false),
        }
    }

    fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        if let (Self::Valid(ref info), Self::Valid(ref other)) = (self, other) {
            info.update_context(other, ctx)
        } else {
            Ok(())
        }
    }

    fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Valid(info) => info.apply_context(ctx).map(Self::Valid),
            Self::Invalid => Ok(Self::Invalid),
        }
    }
}
