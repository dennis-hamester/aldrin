use super::{Receive, Send};
use crate::bus_listener::BusListenerScope;
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, StartBusListener, StartBusListenerReply, StartBusListenerResult};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct StartBusListenerStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
    pub scope: BusListenerScope,
}

impl StartBusListenerStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to start bus listener for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::StartBusListener(StartBusListener {
                serial: serial.clone(),
                cookie: self.cookie.clone(),
                scope: self.scope,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send start-bus-listener message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::StartBusListenerReply(StartBusListenerReply {
                serial,
                result: StartBusListenerResult::Ok,
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive start-bus-listener-reply message"))?;

        Ok(())
    }
}
