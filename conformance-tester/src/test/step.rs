use super::ConnectClient;
use crate::broker::Broker;
use crate::context::Context;
use anyhow::Result;
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Step {
    Connect(ConnectClient),
}

impl Step {
    pub async fn run(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        match self {
            Self::Connect(step) => step.run(broker, ctx, timeout).await,
        }
    }
}
