mod close_invalid_receiver;
mod close_invalid_sender;
mod connect_ok;
mod connect_version_mismatch;
mod create_channel_with_claimed_receiver;
mod create_channel_with_claimed_sender;
mod shutdown_by_broker;
mod shutdown_by_client;

use super::{BrokerRunArgs, BrokerUnderTest};
use crate::test::{MessageType, RunBox, RunError, Test};
use anyhow::{anyhow, Result};
use std::future::Future;
use std::time::Duration;
use tokio::time::{self, Instant};

pub fn make_tests() -> Vec<BrokerTest> {
    vec![
        close_invalid_receiver::make_test(),
        close_invalid_sender::make_test(),
        connect_ok::make_test(),
        connect_version_mismatch::make_test(),
        create_channel_with_claimed_receiver::make_test(),
        create_channel_with_claimed_sender::make_test(),
        shutdown_by_broker::make_test(),
        shutdown_by_client::make_test(),
    ]
}

async fn run_test<F>(test: F, args: BrokerRunArgs) -> Result<(), RunError>
where
    F: for<'a> RunHelper<'a>,
{
    let timeout = Instant::now() + Duration::from_millis(args.timeout);

    let mut broker = time::timeout_at(timeout, BrokerUnderTest::new(args))
        .await
        .map_err(|_| RunError::bare(anyhow!("timeout while starting broker")))??;

    let res = match time::timeout_at(timeout, test.call(&mut broker)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow!("test timed out")),
    };

    broker.result(res).await
}

pub struct BrokerTest {
    name: &'static str,
    short: &'static str,
    long: Option<&'static str>,
    message_types: &'static [MessageType],
    run: Option<RunBox<BrokerRunArgs>>,
}

impl BrokerTest {
    fn new<F>(
        name: &'static str,
        short: &'static str,
        long: Option<&'static str>,
        message_types: &'static [MessageType],
        run: F,
    ) -> Self
    where
        F: for<'a> RunHelper<'a> + 'static,
    {
        BrokerTest {
            name,
            short,
            long,
            message_types,
            run: Some(Box::new(move |args| Box::pin(run_test(run, args)))),
        }
    }
}

impl Test for BrokerTest {
    type Args = BrokerRunArgs;

    fn name(&self) -> &'static str {
        self.name
    }

    fn short(&self) -> &'static str {
        self.short
    }

    fn long(&self) -> Option<&'static str> {
        self.long
    }

    fn message_types(&self) -> &[MessageType] {
        self.message_types
    }

    fn run(&mut self) -> Option<RunBox<BrokerRunArgs>> {
        self.run.take()
    }
}

trait RunHelper<'a>: Send {
    type Future: Future<Output = Result<()>> + Send + 'a;

    fn call(self, args: &'a mut BrokerUnderTest) -> Self::Future;
}

impl<'a, F, Fut> RunHelper<'a> for F
where
    F: FnOnce(&'a mut BrokerUnderTest) -> Fut + Send,
    Fut: Future<Output = Result<()>> + Send + 'a,
{
    type Future = Fut;

    fn call(self, args: &'a mut BrokerUnderTest) -> Self::Future {
        self(args)
    }
}
