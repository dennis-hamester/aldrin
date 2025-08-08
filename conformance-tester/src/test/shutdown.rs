use super::{ConnectionClosed, ReceiveDiscardUntil, RemoveClient, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, Shutdown};
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ShutdownStep {
    #[serde(default)]
    pub client: ClientId,
}

impl ShutdownStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to shut down client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let send = Send {
            client: self.client.clone(),
            message: Message::Shutdown(Shutdown),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send shutdown message"))?;

        let receive_discard_until = ReceiveDiscardUntil {
            client: self.client.clone(),
            message: Message::Shutdown(Shutdown),
        };
        receive_discard_until
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive shutdown message"))?;

        let connection_closed = ConnectionClosed {
            client: self.client.clone(),
        };
        connection_closed.run(ctx, timeout).await?;

        let remove_client = RemoveClient {
            client: self.client.clone(),
        };
        remove_client.run(ctx, timeout).await?;

        Ok(())
    }
}
