use crate::client_id::ClientId;
use crate::context::Context;
use crate::message::Message;
use crate::util::FutureExt;
use anyhow::{Error, Result, anyhow};
use serde::Deserialize;
use std::fmt::Write;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ReceiveUnordered {
    #[serde(default)]
    pub client: ClientId,

    pub messages: Vec<Message>,
}

impl ReceiveUnordered {
    pub(crate) async fn run(&self, ctx: &mut Context, timeout: Instant) -> Result<()> {
        let mut received = Vec::new();
        let mut missing = Vec::from_iter(&self.messages);

        'outer: while !missing.is_empty() {
            let client = ctx.get_client_mut(&self.client)?;

            let proto_msg = client
                .receive()
                .timeout_at(timeout)
                .await
                .map_err(|_| {
                    self.generate_error(
                        &received,
                        &missing,
                        ctx,
                        anyhow!("timeout while receiving message"),
                    )
                })?
                .map_err(|e| {
                    self.generate_error(
                        &received,
                        &missing,
                        ctx,
                        e.context(anyhow!("failed to receive message")),
                    )
                })?;

            let msg = proto_msg.try_into()?;

            for (i, missing_msg) in missing.iter().enumerate() {
                if missing_msg.matches(&msg, ctx)? {
                    missing_msg.update_context(&msg, ctx)?;
                    missing.swap_remove(i);
                    received.push(msg);
                    continue 'outer;
                }
            }

            let msg = serde_json::to_string_pretty(&msg).unwrap();
            return Err(self.generate_error(
                &received,
                &missing,
                ctx,
                anyhow!("received unexpected message\n{msg}"),
            ));
        }

        Ok(())
    }

    fn generate_error(
        &self,
        received: &[Message],
        missing: &[&Message],
        ctx: &Context,
        err: impl Into<Error>,
    ) -> Error {
        let mut error_msg = format!(
            "failed to receive {} unordered message(s) for client `{}`",
            received.len() + missing.len(),
            self.client
        );

        for (i, msg) in missing.iter().enumerate() {
            let msg = match msg.apply_context(ctx) {
                Ok(msg) => serde_json::to_string_pretty(&msg).unwrap(),
                Err(e) => return e,
            };

            let _ = write!(error_msg, "\n\nmissing message {}:\n{msg}", i + 1);
        }

        for (i, msg) in received.iter().enumerate() {
            let msg = match msg.apply_context(ctx) {
                Ok(msg) => serde_json::to_string_pretty(&msg).unwrap(),
                Err(e) => return e,
            };

            let _ = write!(error_msg, "\n\nreceived message {}:\n{msg}", i + 1);
        }

        err.into().context(error_msg)
    }
}
