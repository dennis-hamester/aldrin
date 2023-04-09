use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{ItemReceived, Message, SendItem};
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SendItemStep {
    #[serde(default)]
    pub client: ClientId,
    pub cookie: UuidRef,
    pub receiver: Option<ClientId>,

    #[serde(flatten)]
    pub value: Value,
}

impl SendItemStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send item for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let send = Send {
            client: self.client.clone(),
            message: Message::SendItem(SendItem {
                cookie: self.cookie.clone(),
                value: self.value.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send send-item message"))?;

        let receiver = self.receiver.clone().unwrap_or_else(|| self.client.clone());

        let receive = Receive {
            client: receiver.clone(),
            message: Message::ItemReceived(ItemReceived {
                cookie: self.cookie.clone(),
                value: self.value.clone(),
            }),
        };
        receive.run(ctx, timeout).await.with_context(|| {
            anyhow!("failed to receive item-received message for client `{receiver}`")
        })?;

        Ok(())
    }
}
