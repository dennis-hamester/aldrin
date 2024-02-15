use super::{Receive, Send};
use crate::broker::Broker;
use crate::client::Client;
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Connect, Connect2, ConnectReply, ConnectReply2, ConnectResult, Message};
use aldrin_core::ProtocolVersion;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConnectClient {
    #[serde(default)]
    pub client: ClientId,

    #[serde(default = "default_true")]
    pub handshake: bool,

    #[serde(default = "default_true")]
    pub sync: bool,

    #[serde(default = "default_true")]
    pub shutdown: bool,
}

fn default_true() -> bool {
    true
}

impl ConnectClient {
    pub async fn run(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(broker, ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to connect client `{}`", self.client))
    }

    async fn run_impl(&self, broker: &Broker, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let client = Client::connect(broker.port(), timeout, self.sync, self.shutdown).await?;
        ctx.set_client(self.client.clone(), client)?;

        if self.handshake {
            let version = ctx.version();

            if version == ProtocolVersion::V1_14 {
                self.connect(ctx, timeout).await?;
            } else {
                self.connect2(ctx, timeout).await?;
            }
        }

        Ok(())
    }

    async fn connect(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let send = Send {
            client: self.client.clone(),
            message: Message::Connect(Connect { version: 14 }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send connect message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::ConnectReply(ConnectReply::Ok),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive connect-reply message"))?;

        Ok(())
    }

    async fn connect2(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let version = ctx.version();

        let send = Send {
            client: self.client.clone(),
            message: Message::Connect2(Connect2 {
                major_version: version.major(),
                minor_version: version.minor(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send connect2 message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::ConnectReply2(ConnectReply2 {
                result: ConnectResult::Ok {
                    minor_version: version.minor(),
                },
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive connect-reply2 message"))?;

        Ok(())
    }
}
