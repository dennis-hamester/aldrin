mod claim_channel_end;
mod close_channel;
mod close_channel_end;
mod connect_client;
mod connection_closed;
mod create_bus_listener;
mod create_channel;
mod create_object;
mod create_service;
mod create_service2;
mod destroy_bus_listener;
mod destroy_object;
mod destroy_service;
mod receive;
mod receive_discard_until;
mod receive_unordered;
mod remove_client;
mod send;
mod send_item;
mod shutdown;
mod start_bus_listener;
mod step;
mod stop_bus_listener;
mod subscribe_all_events;
mod subscribe_event;
mod sync;
mod unsubscribe_event;

use crate::broker::Broker;
use crate::context::Context;
use crate::message_type::MessageType;
use crate::protocol_version_serde;
use aldrin_core::ProtocolVersion;
use anyhow::{anyhow, Context as _, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::Path;
use tokio::time::Instant;

pub(crate) use claim_channel_end::ClaimChannelEndStep;
pub(crate) use close_channel::CloseChannel;
pub(crate) use close_channel_end::CloseChannelEndStep;
pub(crate) use connect_client::ConnectClient;
pub(crate) use connection_closed::ConnectionClosed;
pub(crate) use create_bus_listener::CreateBusListenerStep;
pub(crate) use create_channel::CreateChannelStep;
pub(crate) use create_object::CreateObjectStep;
pub(crate) use create_service::CreateServiceStep;
pub(crate) use create_service2::CreateService2Step;
pub(crate) use destroy_bus_listener::DestroyBusListenerStep;
pub(crate) use destroy_object::DestroyObjectStep;
pub(crate) use destroy_service::DestroyServiceStep;
pub(crate) use receive::Receive;
pub(crate) use receive_discard_until::ReceiveDiscardUntil;
pub(crate) use receive_unordered::ReceiveUnordered;
pub(crate) use remove_client::RemoveClient;
pub(crate) use send::Send;
pub(crate) use send_item::SendItemStep;
pub(crate) use shutdown::ShutdownStep;
pub(crate) use start_bus_listener::StartBusListenerStep;
pub(crate) use step::Step;
pub(crate) use stop_bus_listener::StopBusListenerStep;
pub(crate) use subscribe_all_events::SubscribeAllEventsStep;
pub(crate) use subscribe_event::SubscribeEventStep;
pub(crate) use sync::SyncStep;
pub(crate) use unsubscribe_event::UnsubscribeEventStep;

pub(crate) static BUILT_IN_TESTS: Lazy<Vec<Test>> = Lazy::new(|| {
    let sources = [
        include_str!("../tests/abort-call-1.json"),
        include_str!("../tests/abort-call-2.json"),
        include_str!("../tests/abort-call-by-disconnect.json"),
        include_str!("../tests/abort-call-old-callee.json"),
        include_str!("../tests/abort-call-old-version.json"),
        include_str!("../tests/abort-invalid-call.json"),
        include_str!("../tests/call-function-aborted.json"),
        include_str!("../tests/call-function-err.json"),
        include_str!("../tests/call-function-invalid-args.json"),
        include_str!("../tests/call-function-invalid-service.json"),
        include_str!("../tests/call-function-ok.json"),
        include_str!("../tests/call-function.json"),
        include_str!("../tests/call-function2-denied.json"),
        include_str!("../tests/call-function2-with-version-new-client.json"),
        include_str!("../tests/call-function2-with-version-old-client.json"),
        include_str!("../tests/call-function2-without-version-new-client.json"),
        include_str!("../tests/call-function2-without-version-old-client.json"),
        include_str!("../tests/call-invalid-function.json"),
        include_str!("../tests/channel-capacity-overflow-1.json"),
        include_str!("../tests/channel-capacity-overflow-2.json"),
        include_str!("../tests/claim-invalid-receiver.json"),
        include_str!("../tests/claim-invalid-sender.json"),
        include_str!("../tests/claim-receiver-after-close.json"),
        include_str!("../tests/claim-receiver-already-claimed.json"),
        include_str!("../tests/claim-receiver-ok.json"),
        include_str!("../tests/claim-sender-after-close.json"),
        include_str!("../tests/claim-sender-already-claimed.json"),
        include_str!("../tests/claim-sender-ok.json"),
        include_str!("../tests/close-foreign-receiver.json"),
        include_str!("../tests/close-foreign-sender.json"),
        include_str!("../tests/close-invalid-receiver.json"),
        include_str!("../tests/close-invalid-sender.json"),
        include_str!("../tests/close-receiver-ok.json"),
        include_str!("../tests/close-receiver-with-sender-claimed.json"),
        include_str!("../tests/close-sender-ok.json"),
        include_str!("../tests/close-sender-with-receiver-claimed.json"),
        include_str!("../tests/connect-15-on-14.json"),
        include_str!("../tests/connect-and-disconnect.json"),
        include_str!("../tests/connect-and-shutdown.json"),
        include_str!("../tests/connect-ok.json"),
        include_str!("../tests/connect-version-too-high.json"),
        include_str!("../tests/connect-version-too-low.json"),
        include_str!("../tests/connect2-14-on-15.json"),
        include_str!("../tests/connect2-incompatible-major.json"),
        include_str!("../tests/connect2-ok.json"),
        include_str!("../tests/create-bus-listener.json"),
        include_str!("../tests/create-channel-receiver.json"),
        include_str!("../tests/create-channel-sender.json"),
        include_str!("../tests/create-object-duplicate.json"),
        include_str!("../tests/create-object-ok.json"),
        include_str!("../tests/create-service-duplicate.json"),
        include_str!("../tests/create-service-foreign-object.json"),
        include_str!("../tests/create-service-invalid-object.json"),
        include_str!("../tests/create-service-ok.json"),
        include_str!("../tests/create-service2-invalid-info.json"),
        include_str!("../tests/create-service2-ok.json"),
        include_str!("../tests/destroy-bus-listener.json"),
        include_str!("../tests/destroy-foreign-bus-listener.json"),
        include_str!("../tests/destroy-foreign-object.json"),
        include_str!("../tests/destroy-foreign-service.json"),
        include_str!("../tests/destroy-invalid-bus-listener.json"),
        include_str!("../tests/destroy-invalid-object.json"),
        include_str!("../tests/destroy-invalid-service.json"),
        include_str!("../tests/destroy-object-ok.json"),
        include_str!("../tests/destroy-service-ok.json"),
        include_str!("../tests/emit-event-0-subscribers.json"),
        include_str!("../tests/emit-event-1-subscriber.json"),
        include_str!("../tests/emit-event-2-subscribers.json"),
        include_str!("../tests/emit-event-foreign-service.json"),
        include_str!("../tests/invalid-message.json"),
        include_str!("../tests/one-bus-event-per-client.json"),
        include_str!("../tests/query-invalid-service-version.json"),
        include_str!("../tests/query-service-version-ok.json"),
        include_str!("../tests/resubscribe-event.json"),
        include_str!("../tests/send-item-with-unclaimed-receiver.json"),
        include_str!("../tests/send-item-without-capacity.json"),
        include_str!("../tests/send-item.json"),
        include_str!("../tests/shutdown-with-all-events-subscribed.json"),
        include_str!("../tests/start-bus-listener-already-started.json"),
        include_str!("../tests/start-bus-listener-ok.json"),
        include_str!("../tests/start-foreign-bus-listener.json"),
        include_str!("../tests/start-invalid-bus-listener.json"),
        include_str!("../tests/stop-bus-listener-not-started-1.json"),
        include_str!("../tests/stop-bus-listener-not-started-2.json"),
        include_str!("../tests/stop-bus-listener-ok.json"),
        include_str!("../tests/stop-foreign-bus-listener.json"),
        include_str!("../tests/stop-invalid-bus-listener.json"),
        include_str!("../tests/subscribe-all-events-not-supported.json"),
        include_str!("../tests/subscribe-all-events-ok.json"),
        include_str!("../tests/subscribe-event-destroy.json"),
        include_str!("../tests/subscribe-event-invalid-service.json"),
        include_str!("../tests/subscribe-event-ok.json"),
        include_str!("../tests/subscribe-event-twice.json"),
        include_str!("../tests/subscribe-invalid-service.json"),
        include_str!("../tests/subscribe-service-ok.json"),
        include_str!("../tests/sync.json"),
        include_str!("../tests/unsubscribe-all-events-1.json"),
        include_str!("../tests/unsubscribe-all-events-2.json"),
        include_str!("../tests/unsubscribe-event.json"),
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

pub(crate) fn get_tests(custom: Option<&Path>) -> Result<Cow<'_, [Test]>> {
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
pub(crate) struct Test {
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,

    #[serde(default)]
    pub message_types: BTreeSet<MessageType>,

    #[serde(
        deserialize_with = "protocol_version_serde::deserialize",
        default = "default_version"
    )]
    pub version: ProtocolVersion,

    pub steps: Vec<Step>,
}

impl Test {
    pub(crate) async fn run(
        &self,
        broker: &Broker,
        timeout: Instant,
        version: ProtocolVersion,
    ) -> Result<()> {
        let mut ctx = Context::new(version);

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

const fn default_version() -> ProtocolVersion {
    ProtocolVersion::V1_14
}
