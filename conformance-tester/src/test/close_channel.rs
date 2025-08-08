use super::CloseChannelEndStep;
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::ChannelEnd;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CloseChannel {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
    pub sender: Option<ClientId>,
    pub receiver: Option<ClientId>,
}

impl CloseChannel {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to close channel"))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let sender = self.sender.clone().unwrap_or_else(|| self.client.clone());
        let receiver = self.receiver.clone().unwrap_or_else(|| self.client.clone());

        let close_channel_end = CloseChannelEndStep {
            client: sender.clone(),
            serial: self.serial.clone(),
            cookie: self.cookie.clone(),
            end: ChannelEnd::Sender,
            with_other: true,
            other: Some(receiver.clone()),
        };
        close_channel_end.run(ctx, timeout).await?;

        let close_channel_end = CloseChannelEndStep {
            client: receiver.clone(),
            serial: self.serial.clone(),
            cookie: self.cookie.clone(),
            end: ChannelEnd::Receiver,
            with_other: false,
            other: None,
        };
        close_channel_end.run(ctx, timeout).await?;

        Ok(())
    }
}
