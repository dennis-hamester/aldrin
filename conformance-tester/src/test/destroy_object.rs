use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{DestroyObject, DestroyObjectReply, DestroyObjectResult, Message};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DestroyObjectStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
}

impl DestroyObjectStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to destroy object for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::DestroyObject(DestroyObject {
                serial: serial.clone(),
                cookie: self.cookie.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send destroy-object message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::DestroyObjectReply(DestroyObjectReply {
                serial,
                result: DestroyObjectResult::Ok,
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive destroy-object-reply message"))?;

        Ok(())
    }
}
