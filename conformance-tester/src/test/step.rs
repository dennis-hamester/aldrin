use super::{
    ClaimChannelEndStep, CloseChannel, CloseChannelEndStep, ConnectClient, ConnectionClosed,
    CreateChannelStep, CreateObjectStep, CreateServiceStep, DestroyObjectStep, DestroyServiceStep,
    Receive, ReceiveDiscardUntil, ReceiveUnordered, RemoveClient, Send, SendItemStep, ShutdownStep,
    SubscribeEventStep, SyncStep, UnsubscribeEventStep,
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
    CreateService(CreateServiceStep),
    DestroyService(DestroyServiceStep),
    SubscribeEvent(SubscribeEventStep),
    UnsubscribeEvent(UnsubscribeEventStep),
    CreateChannel(CreateChannelStep),
    CloseChannelEnd(CloseChannelEndStep),
    CloseChannel(CloseChannel),
    ClaimChannelEnd(ClaimChannelEndStep),
    SendItem(SendItemStep),
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
            Self::CreateService(step) => step.run(ctx, timeout).await,
            Self::DestroyService(step) => step.run(ctx, timeout).await,
            Self::SubscribeEvent(step) => step.run(ctx, timeout).await,
            Self::UnsubscribeEvent(step) => step.run(ctx, timeout).await,
            Self::CreateChannel(step) => step.run(ctx, timeout).await,
            Self::CloseChannelEnd(step) => step.run(ctx, timeout).await,
            Self::CloseChannel(step) => step.run(ctx, timeout).await,
            Self::ClaimChannelEnd(step) => step.run(ctx, timeout).await,
            Self::SendItem(step) => step.run(ctx, timeout).await,
            Self::Sync(step) => step.run(ctx, timeout).await,
        }
    }
}
