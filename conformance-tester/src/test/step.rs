use super::{ConnectClient, ConnectionClosed, Receive, RemoveClient, Send, SyncStep};
use crate::broker::Broker;
use crate::context::Context;
use anyhow::Result;
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Step {
    Connect(ConnectClient),
    RemoveClient(RemoveClient),
    Receive(Receive),
    Send(Send),
    ConnectionClosed(ConnectionClosed),
    Sync(SyncStep),
}

impl Step {
    pub async fn run(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        match self {
            Self::Connect(step) => step.run(broker, ctx, timeout).await,
            Self::RemoveClient(step) => step.run(ctx, timeout).await,
            Self::Receive(step) => step.run(ctx, timeout).await,
            Self::Send(step) => step.run(ctx, timeout).await,
            Self::ConnectionClosed(step) => step.run(ctx, timeout).await,
            Self::Sync(step) => step.run(ctx, timeout).await,
        }
    }
}
