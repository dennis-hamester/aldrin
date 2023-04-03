use super::{
    ConnectClient, ConnectionClosed, CreateObjectStep, DestroyObjectStep, Receive,
    ReceiveDiscardUntil, ReceiveUnordered, RemoveClient, Send, ShutdownStep, SyncStep,
};
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
    Shutdown(ShutdownStep),
    Receive(Receive),
    ReceiveUnordered(ReceiveUnordered),
    ReceiveDiscardUntil(ReceiveDiscardUntil),
    Send(Send),
    ConnectionClosed(ConnectionClosed),
    CreateObject(CreateObjectStep),
    DestroyObject(DestroyObjectStep),
    Sync(SyncStep),
}

impl Step {
    pub async fn run(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        match self {
            Self::Connect(step) => step.run(broker, ctx, timeout).await,
            Self::RemoveClient(step) => step.run(ctx, timeout).await,
            Self::Shutdown(step) => step.run(ctx, timeout).await,
            Self::Receive(step) => step.run(ctx, timeout).await,
            Self::ReceiveUnordered(step) => step.run(ctx, timeout).await,
            Self::ReceiveDiscardUntil(step) => step.run(ctx, timeout).await,
            Self::Send(step) => step.run(ctx, timeout).await,
            Self::ConnectionClosed(step) => step.run(ctx, timeout).await,
            Self::CreateObject(step) => step.run(ctx, timeout).await,
            Self::DestroyObject(step) => step.run(ctx, timeout).await,
            Self::Sync(step) => step.run(ctx, timeout).await,
        }
    }
}
