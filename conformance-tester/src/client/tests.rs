mod connect_ok;
mod connect_version_mismatch;
mod shutdown_by_broker;
mod shutdown_by_client;

use super::{ClientRunArgs, ClientUnderTest};
use crate::test::{MessageType, RunBox, RunError, Test};
use anyhow::{anyhow, Result};
use std::future::Future;
use std::time::Duration;
use tokio::time;

pub fn make_tests() -> Vec<ClientTest> {
    vec![
        connect_ok::make_test(),
        connect_version_mismatch::make_test(),
        shutdown_by_broker::make_test(),
        shutdown_by_client::make_test(),
    ]
}

async fn run_test<F>(test: F, args: ClientRunArgs) -> Result<(), RunError>
where
    F: for<'a> RunHelper<'a>,
{
    let timeout = Duration::from_millis(args.timeout);
    let mut client = ClientUnderTest::new(args).await?;
    let res = match time::timeout(timeout, test.call(&mut client)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow!("test timed out")),
    };
    client.result(res).await
}

pub struct ClientTest {
    name: &'static str,
    short: &'static str,
    long: Option<&'static str>,
    message_types: &'static [MessageType],
    run: Option<RunBox<ClientRunArgs>>,
}

impl ClientTest {
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
        ClientTest {
            name,
            short,
            long,
            message_types,
            run: Some(Box::new(move |args| Box::pin(run_test(run, args)))),
        }
    }
}

impl Test for ClientTest {
    type Args = ClientRunArgs;

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

    fn run(&mut self) -> Option<RunBox<ClientRunArgs>> {
        self.run.take()
    }
}

trait RunHelper<'a>: Send {
    type Future: Future<Output = Result<()>> + Send + 'a;

    fn call(self, args: &'a mut ClientUnderTest) -> Self::Future;
}

impl<'a, F, Fut> RunHelper<'a> for F
where
    F: FnOnce(&'a mut ClientUnderTest) -> Fut + Send,
    Fut: Future<Output = Result<()>> + Send + 'a,
{
    type Future = Fut;

    fn call(self, args: &'a mut ClientUnderTest) -> Self::Future {
        self(args)
    }
}
