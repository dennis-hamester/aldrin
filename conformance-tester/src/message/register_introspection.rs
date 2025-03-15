use crate::context::Context;
use aldrin_core::{message, SerializedValue};
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RegisterIntrospection {
    pub type_ids: HashSet<Uuid>,
}

impl RegisterIntrospection {
    pub fn to_core(&self, _ctx: &Context) -> Result<message::RegisterIntrospection> {
        let value = SerializedValue::serialize(&self.type_ids)
            .with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::RegisterIntrospection { value })
    }

    pub fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        Ok(self.type_ids == other.type_ids)
    }

    pub fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::RegisterIntrospection> for RegisterIntrospection {
    type Error = Error;

    fn try_from(msg: message::RegisterIntrospection) -> Result<Self> {
        let type_ids = msg.value.deserialize().with_context(|| {
            anyhow!("failed to deserialize type ids from register-introspection message")
        })?;

        Ok(Self { type_ids })
    }
}
