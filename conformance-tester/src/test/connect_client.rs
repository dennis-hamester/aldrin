use super::{Receive, Send};
use crate::broker::Broker;
use crate::client::Client;
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Connect, ConnectReply, Message};
use crate::value::Value;
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
        let client = Client::connect(broker.port(), timeout, self.sync).await?;
        ctx.set_client(self.client.clone(), client)?;

        if self.handshake {
            let send = Send {
                client: self.client.clone(),
                message: Message::Connect(Connect {
                    version: aldrin_proto::VERSION,
                    value: Value::None,
                }),
            };
            send.run(ctx, timeout)
                .await
                .with_context(|| anyhow!("failed to send connect message"))?;

            let receive = Receive {
                client: self.client.clone(),
                message: Message::ConnectReply(ConnectReply::Ok {
                    value: Value::Ignore,
                }),
            };
            receive
                .run(ctx, timeout)
                .await
                .with_context(|| anyhow!("failed to receive connect-reply message"))?;
        }

        Ok(())
    }
}
