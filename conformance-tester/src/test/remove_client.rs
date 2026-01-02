use crate::client_id::ClientId;
use crate::context::Context;
use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RemoveClient {
    #[serde(default)]
    pub client: ClientId,
}

impl RemoveClient {
    pub(crate) fn run(&self, ctx: &mut Context, _timeout: Instant) -> Result<()> {
        ctx.remove_client(&self.client)
            .with_context(|| anyhow!("failed to remove client `{}`", self.client))
    }
}
