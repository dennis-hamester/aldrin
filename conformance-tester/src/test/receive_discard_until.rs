use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::Message;
use crate::util::FutureExt;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ReceiveDiscardUntil {
    #[serde(default)]
    pub client: ClientId,

    #[serde(flatten)]
    pub message: Message,
}

impl ReceiveDiscardUntil {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive a message for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        loop {
            let client = ctx.get_client_mut(&self.client)?;

            let proto_msg = client
                .receive()
                .timeout_at(timeout)
                .await
                .map_err(|_| anyhow!("timeout while receiving message"))??;

            let msg = proto_msg.try_into()?;

            if self.message.matches(&msg, ctx)? {
                self.message.update_context(&msg, ctx)?;
                break Ok(());
            }
        }
    }
}
