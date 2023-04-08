use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, UnsubscribeEvent};
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnsubscribeEventStep {
    #[serde(default)]
    pub client: ClientId,

    pub service_cookie: UuidRef,
    pub event: u32,

    #[serde(default = "default_true")]
    pub with_owner: bool,

    pub owner: Option<ClientId>,
}

fn default_true() -> bool {
    true
}

impl UnsubscribeEventStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout).await.with_context(|| {
            anyhow!(
                "failed to unsubscribe from event {} for client `{}`",
                self.event,
                self.client
            )
        })
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let send = Send {
            client: self.client.clone(),
            message: Message::UnsubscribeEvent(UnsubscribeEvent {
                service_cookie: self.service_cookie.clone(),
                event: self.event,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send unsubscribe-event message"))?;

        if self.with_owner {
            let owner = self.owner.clone().unwrap_or_else(|| self.client.clone());

            let receive = Receive {
                client: owner.clone(),
                message: Message::UnsubscribeEvent(UnsubscribeEvent {
                    service_cookie: self.service_cookie.clone(),
                    event: self.event,
                }),
            };
            receive.run(ctx, timeout).await.with_context(|| {
                anyhow!("failed to receive unsubscribe-event message for client `{owner}`")
            })?;
        }

        Ok(())
    }
}
