use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, Sync, SyncReply};
use crate::serial::Serial;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SyncStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
}

impl SyncStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to synchronize client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::Sync(Sync {
                serial: serial.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send sync message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::SyncReply(SyncReply { serial }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive sync-reply message"))?;

        Ok(())
    }
}
