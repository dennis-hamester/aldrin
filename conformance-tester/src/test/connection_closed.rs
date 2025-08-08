use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::Message;
use crate::util::FutureExt;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConnectionClosed {
    #[serde(default)]
    pub client: ClientId,
}

impl ConnectionClosed {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("the connection of client `{}` did not close", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let client = ctx.get_client_mut(&self.client)?;

        let res = client
            .expect_closed()
            .timeout_at(timeout)
            .await
            .map_err(|_| anyhow!("timeout while waiting for the connection to close"))?;

        match res {
            Ok(()) => Ok(()),

            Err(Ok(msg)) => {
                let msg = serde_json::to_string_pretty(&Message::try_from(msg)?).unwrap();
                Err(anyhow!("received a message\n\n{msg}"))
            }

            Err(Err(e)) => Err(e),
        }
    }
}
