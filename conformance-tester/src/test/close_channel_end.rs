use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{
    ChannelEnd, ChannelEndClosed, CloseChannelEnd, CloseChannelEndReply, CloseChannelEndResult,
    Message,
};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CloseChannelEndStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
    pub end: ChannelEnd,

    #[serde(default = "default_true")]
    pub with_other: bool,

    pub other: Option<ClientId>,
}

fn default_true() -> bool {
    true
}

impl CloseChannelEndStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to close {} for client `{}`", self.end, self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::CloseChannelEnd(CloseChannelEnd {
                serial: serial.clone(),
                cookie: self.cookie.clone(),
                end: self.end,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send close-channel-end message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::CloseChannelEndReply(CloseChannelEndReply {
                serial,
                result: CloseChannelEndResult::Ok,
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive close-channel-end-reply message"))?;

        if self.with_other {
            let other = self.other.clone().unwrap_or_else(|| self.client.clone());

            let receive = Receive {
                client: other.clone(),
                message: Message::ChannelEndClosed(ChannelEndClosed {
                    cookie: self.cookie.clone(),
                    end: self.end,
                }),
            };
            receive.run(ctx, timeout).await.with_context(|| {
                anyhow!("failed to receive channel-end-closed message for `{other}`")
            })?;
        }

        Ok(())
    }
}
