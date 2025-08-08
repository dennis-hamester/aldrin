use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{CreateService, CreateServiceReply, CreateServiceResult, Message};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CreateServiceStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub object_cookie: UuidRef,
    pub service_uuid: UuidRef,
    pub service_cookie: UuidRef,
    pub version: u32,
}

impl CreateServiceStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to create service for client `{}`", self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::CreateService(CreateService {
                serial: serial.clone(),
                object_cookie: self.object_cookie.clone(),
                uuid: self.service_uuid.clone(),
                version: self.version,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send create-service message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::CreateServiceReply(CreateServiceReply {
                serial,
                result: CreateServiceResult::Ok {
                    cookie: self.service_cookie.clone(),
                },
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive create-service-reply message"))?;

        Ok(())
    }
}
