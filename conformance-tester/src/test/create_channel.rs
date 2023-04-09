use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{ChannelEnd, CreateChannel, CreateChannelReply, Message};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateChannelStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub claim: ChannelEnd,
    pub cookie: UuidRef,
}

impl CreateChannelStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to create channel for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::CreateChannel(CreateChannel {
                serial: serial.clone(),
                claim: self.claim,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send create-channel message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::CreateChannelReply(CreateChannelReply {
                serial,
                cookie: self.cookie.clone(),
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive create-channel-reply message"))?;

        Ok(())
    }
}
