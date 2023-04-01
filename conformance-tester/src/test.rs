mod connect_client;
mod connection_closed;
mod receive;
mod receive_discard_until;
mod remove_client;
mod send;
mod shutdown;
mod step;
mod sync;

use crate::broker::Broker;
use crate::context::Context;
use crate::message_type::MessageType;
use anyhow::{anyhow, Context as _, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::Path;
use tokio::time::Instant;

pub use connect_client::ConnectClient;
pub use connection_closed::ConnectionClosed;
pub use receive::Receive;
pub use receive_discard_until::ReceiveDiscardUntil;
pub use remove_client::RemoveClient;
pub use send::Send;
pub use shutdown::ShutdownStep;
pub use step::Step;
pub use sync::SyncStep;

pub static BUILT_IN_TESTS: Lazy<Vec<Test>> = Lazy::new(|| {
    let sources = [
        include_str!("../tests/connect-and-disconnect.json"),
        include_str!("../tests/connect-and-shutdown.json"),
        include_str!("../tests/connect-ok.json"),
        include_str!("../tests/connect-version-too-high.json"),
        include_str!("../tests/connect-version-too-low.json"),
        include_str!("../tests/sync.json"),
    ];

    let mut tests: Vec<Test> = sources
        .into_iter()
        .map(|src| {
            serde_json::from_str(src)
                .with_context(|| anyhow!("failed to parse built-in test:\n{src}"))
                .unwrap()
        })
        .collect();

    tests.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    tests
});

pub fn get_tests(custom: Option<&Path>) -> Result<Cow<[Test]>> {
    let Some(custom) = custom else {
        return Ok(Cow::Borrowed(&*BUILT_IN_TESTS));
    };

    let metadata = fs::metadata(custom).with_context(|| {
        anyhow!(
            "failed to determine if `{}` is a file or directory",
            custom.display()
        )
    })?;

    if metadata.is_file() {
        let file =
            File::open(custom).with_context(|| anyhow!("failed to open `{}`", custom.display()))?;
        let test: Test = serde_json::from_reader(file)
            .with_context(|| anyhow!("failed to parse `{}`", custom.display()))?;

        Ok(Cow::Owned(vec![test]))
    } else {
        let read_dir = fs::read_dir(custom)
            .with_context(|| anyhow!("failed to read directory `{}`", custom.display()))?;

        let mut tests = BTreeMap::new();
        for entry in read_dir {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
            if path.extension() != Some(OsStr::new("json")) {
                continue;
            }

            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            let file = File::open(&path)
                .with_context(|| anyhow!("failed to open `{}`", path.display()))?;
            let test: Test = serde_json::from_reader(file)
                .with_context(|| anyhow!("failed to parse `{}`", path.display()))?;

            if let Some(dup) = tests.insert(test.name.clone(), test) {
                return Err(anyhow!("duplicate test name `{}`", dup.name));
            }
        }

        Ok(Cow::Owned(tests.into_values().collect()))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Test {
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,

    #[serde(default)]
    pub message_types: BTreeSet<MessageType>,

    pub steps: Vec<Step>,
}

impl Test {
    pub async fn run(&self, broker: &Broker, timeout: Instant) -> Result<()> {
        let mut ctx = Context::new();

        for (i, step) in self.steps.iter().enumerate() {
            step.run(broker, &mut ctx, timeout)
                .await
                .with_context(|| anyhow!("test failed at step {}", i + 1))?;
        }

        let clients: Vec<_> = ctx.client_ids().cloned().collect();

        for client in &clients {
            if !ctx.get_client(client).unwrap().sync() {
                continue;
            }

            let sync = SyncStep {
                client: client.clone(),
                serial: None,
            };

            sync.run(&mut ctx, timeout).await.with_context(|| {
                anyhow!("implicit final synchronization of client `{client}` failed")
            })?;
        }

        for client in &clients {
            if !ctx.get_client(client).unwrap().shutdown() {
                continue;
            }

            let shutdown = ShutdownStep {
                client: client.clone(),
            };

            shutdown
                .run(&mut ctx, timeout)
                .await
                .with_context(|| anyhow!("implicit final shutdown of client `{client}` failed"))?;
        }

        Ok(())
    }
}
