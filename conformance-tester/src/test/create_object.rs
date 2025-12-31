use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{CreateObject, CreateObjectReply, CreateObjectResult, Message};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CreateObjectStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub uuid: UuidRef,
    pub cookie: UuidRef,
}

impl CreateObjectStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to create object for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::CreateObject(CreateObject {
                serial: serial.clone(),
                uuid: self.uuid.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send create-object message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::CreateObjectReply(CreateObjectReply {
                serial,
                result: CreateObjectResult::Ok {
                    cookie: self.cookie.clone(),
                },
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive create-object-reply message"))?;

        Ok(())
    }
}
