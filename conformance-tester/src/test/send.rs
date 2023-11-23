use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::Message;
use crate::util::FutureExt;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Send {
    #[serde(default)]
    pub client: ClientId,

    #[serde(flatten)]
    pub message: Message,
}

impl Send {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send a message for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let msg = self.message.to_core(ctx)?;
        let client = ctx.get_client_mut(&self.client)?;

        client
            .send(msg)
            .timeout_at(timeout)
            .await
            .map_err(|_| anyhow!("timeout while sending message"))??;

        Ok(())
    }
}
