use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::Message;
use crate::util::FutureExt;
use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Receive {
    #[serde(default)]
    pub client: ClientId,

    #[serde(flatten)]
    pub message: Message,
}

impl Receive {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive a message for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let client = ctx.get_client_mut(&self.client)?;

        let proto_msg = client
            .receive()
            .timeout_at(timeout)
            .await
            .map_err(|_| anyhow!("timeout while receiving message"))??;

        let msg = proto_msg.try_into()?;

        if self.message.matches(&msg, ctx)? {
            self.message.update_context(&msg, ctx)
        } else {
            let received = serde_json::to_string_pretty(&msg).unwrap();
            let expected = serde_json::to_string_pretty(&self.message.apply_context(ctx)?).unwrap();

            Err(anyhow!(
                "unexpected message received\n\nreceived:\n{received}\n\nexpected:\n{expected}"
            ))
        }
    }
}
