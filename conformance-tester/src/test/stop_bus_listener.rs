use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, StopBusListener, StopBusListenerReply, StopBusListenerResult};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StopBusListenerStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
}

impl StopBusListenerStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to stop bus listener for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::StopBusListener(StopBusListener {
                serial: serial.clone(),
                cookie: self.cookie.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send stop-bus-listener message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::StopBusListenerReply(StopBusListenerReply {
                serial,
                result: StopBusListenerResult::Ok,
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive stop-bus-listener-reply message"))?;

        Ok(())
    }
}
