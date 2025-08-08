use super::{Receive, ReceiveUnordered, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{Message, SubscribeEvent, SubscribeEventReply, SubscribeEventResult};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SubscribeEventStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub service_cookie: UuidRef,
    pub event: u32,

    #[serde(default = "default_true")]
    pub with_owner: bool,

    pub owner: Option<ClientId>,
}

fn default_true() -> bool {
    true
}

impl SubscribeEventStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout).await.with_context(|| {
            anyhow!(
                "failed to subscribe to event {} for client `{}`",
                self.event,
                self.client
            )
        })
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::SubscribeEvent(SubscribeEvent {
                serial: Some(serial.clone()),
                service_cookie: self.service_cookie.clone(),
                event: self.event,
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send subscribe-event message"))?;

        if self.handle_owner(ctx, timeout).await? {
            let receive = Receive {
                client: self.client.clone(),
                message: Message::SubscribeEventReply(SubscribeEventReply {
                    serial,
                    result: SubscribeEventResult::Ok,
                }),
            };
            receive
                .run(ctx, timeout)
                .await
                .with_context(|| anyhow!("failed to receive subscribe-event-reply message"))?;
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
                        Message::SubscribeEvent(SubscribeEvent {
                            serial: None,
                            service_cookie: self.service_cookie.clone(),
                            event: self.event,
                        }),
                        Message::SubscribeEventReply(SubscribeEventReply {
                            serial: self.serial.clone().unwrap_or(Serial::Const(0)),
                            result: SubscribeEventResult::Ok,
                        }),
                    ],
                };
                receive_unordered.run(ctx, timeout).await.with_context(|| {
                    anyhow!(
                        "failed to receive subscribe-event and/or subscribe-event-reply message"
                    )
                })?;

                Ok(false)
            } else {
                let receive = Receive {
                    client: owner.clone(),
                    message: Message::SubscribeEvent(SubscribeEvent {
                        serial: None,
                        service_cookie: self.service_cookie.clone(),
                        event: self.event,
                    }),
                };
                receive
                    .run(ctx, timeout)
                    .await
                    .with_context(|| anyhow!("failed to receive subscribe-event message"))?;

                Ok(true)
            }
        } else {
            Ok(true)
        }
    }
}
