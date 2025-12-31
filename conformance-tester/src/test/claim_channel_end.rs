use super::{Receive, Send};
use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::{
    ChannelEnd, ChannelEndClaimed, ChannelEndWithCapacity, ClaimChannelEnd, ClaimChannelEndReply,
    ClaimChannelEndResult, Message,
};
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ClaimChannelEndStep {
    #[serde(default)]
    pub client: ClientId,

    pub serial: Option<Serial>,
    pub cookie: UuidRef,
    pub end: ChannelEnd,
    pub capacity: u32,

    #[serde(default = "default_true")]
    pub with_other: bool,

    pub other: Option<ClientId>,
}

fn default_true() -> bool {
    true
}

impl ClaimChannelEndStep {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        self.run_impl(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to claim {} for client `{}`", self.end, self.client))
    }

    async fn run_impl(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let serial = self.serial.clone().unwrap_or(Serial::Const(0));

        let send = Send {
            client: self.client.clone(),
            message: Message::ClaimChannelEnd(ClaimChannelEnd {
                serial: serial.clone(),
                cookie: self.cookie.clone(),
                end: match self.end {
                    ChannelEnd::Sender => ChannelEndWithCapacity::Sender,
                    ChannelEnd::Receiver => ChannelEndWithCapacity::Receiver {
                        capacity: self.capacity,
                    },
                },
            }),
        };
        send.run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to send claim-channel-end message"))?;

        let receive = Receive {
            client: self.client.clone(),
            message: Message::ClaimChannelEndReply(ClaimChannelEndReply {
                serial,
                result: match self.end {
                    ChannelEnd::Sender => ClaimChannelEndResult::SenderClaimed {
                        capacity: self.capacity,
                    },

                    ChannelEnd::Receiver => ClaimChannelEndResult::ReceiverClaimed,
                },
            }),
        };
        receive
            .run(ctx, timeout)
            .await
            .with_context(|| anyhow!("failed to receive claim-channel-end-reply message"))?;

        if self.with_other {
            let other = self.other.clone().unwrap_or_else(|| self.client.clone());

            let receive = Receive {
                client: other.clone(),
                message: Message::ChannelEndClaimed(ChannelEndClaimed {
                    cookie: self.cookie.clone(),
                    end: match self.end {
                        ChannelEnd::Sender => ChannelEndWithCapacity::Sender,
                        ChannelEnd::Receiver => ChannelEndWithCapacity::Receiver {
                            capacity: self.capacity,
                        },
                    },
                }),
            };
            receive.run(ctx, timeout).await.with_context(|| {
                anyhow!("failed to receive channel-end-claimed message for `{other}`")
            })?;
        }

        Ok(())
    }
}
