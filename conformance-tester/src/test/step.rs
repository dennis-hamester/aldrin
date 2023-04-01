use crate::broker::Broker;
use crate::context::Context;
use anyhow::Result;
use serde::Deserialize;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Step {}

impl Step {
    pub async fn run(&self, _broker: &Broker, _ctx: &mut Context, _timeout: Instant) -> Result<()> {
        match *self {}
    }
}
