use crate::broker::Broker;
use crate::client::Client;
use crate::client_id::ClientId;
use crate::context::Context;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConnectClient {
    #[serde(default)]
    pub client: ClientId,
}

impl ConnectClient {
    pub async fn run(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(broker, ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to connect client `{}`", self.client))
    }

    async fn run_impl(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let client = Client::connect(broker.port(), timeout).await?;
        ctx.set_client(self.client.clone(), client)?;

        Ok(())
    }
}
