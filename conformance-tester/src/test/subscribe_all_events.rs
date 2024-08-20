use super::{Receive, ReceiveUnordered, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{
    Message, SubscribeAllEvents, SubscribeAllEventsReply, SubscribeAllEventsResult,
};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubscribeAllEventsStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub service_cookie: UuidRef,

    #[serde(default = "default_true")]
    pub with_owner: bool,

    pub owner: Option<ClientId>,
}

fn default_true() -> bool {
    true
}

impl SubscribeAllEventsStep {
    pub async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout).await.with_context(|| {
            anyhow!(
                "failed to subscribe to all events for client `{}`",
                self.client
            )
        })
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::SubscribeAllEvents(SubscribeAllEvents {
                serial: Some(serial.clone()),
                service_cookie: self.service_cookie.clone(),
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send subscribe-all-events message"))?;

        if self.handle_owner(ctx, timeout).await? {
            let receive = Receive {
                client: self.client.clone(),
                message: Message::SubscribeAllEventsReply(SubscribeAllEventsReply {
                    serial,
                    result: SubscribeAllEventsResult::Ok,
                }),
            };
            receive
                .run(ctx, timeout)
                .await
                .with_context(|| anyhow!("failed to receive subscribe-all-events-reply message"))?;
        }

        Ok(())
    }

    async fn handle_owner(&self, ctx: &mut Context, timeout: Instant) -> Result<bool> {
        if self.with_owner {
            let owner = self.owner.clone().unwrap_or_else(|| self.client.clone());

            if self.client == owner {
                let receive_unordered = ReceiveUnordered {
                    client: self.client.clone(),
                    messages: vec![
                        Message::SubscribeAllEvents(SubscribeAllEvents {
                            serial: None,
                            service_cookie: self.service_cookie.clone(),
                        }),
                        Message::SubscribeAllEventsReply(SubscribeAllEventsReply {
                            serial: self.serial.clone().unwrap_or(Serial::Const(0)),
                            result: SubscribeAllEventsResult::Ok,
                        }),
                    ],
                };
                receive_unordered.run(ctx, timeout).await.with_context(|| {
                    anyhow!(
                        "failed to receive subscribe-all-events and/or subscribe-all-events-reply message"
                    )
                })?;

                Ok(false)
            } else {
                let receive = Receive {
                    client: owner.clone(),
                    message: Message::SubscribeAllEvents(SubscribeAllEvents {
                        serial: None,
                        service_cookie: self.service_cookie.clone(),
                    }),
                };
                receive
                    .run(ctx, timeout)
                    .await
                    .with_context(|| anyhow!("failed to receive subscribe-all-events message"))?;

                Ok(true)
            }
        } else {
            Ok(true)
        }
    }
}
