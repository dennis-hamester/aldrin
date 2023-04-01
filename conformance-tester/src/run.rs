use crate::broker::Broker;
use crate::output;
use crate::run_error::RunError;
use crate::test::Test;
use crate::{BrokerRunArgs, RunArgs};
use anyhow::Result;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::thread;
use std::time::Duration;
use termcolor::WriteColor;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::time::Instant;

pub fn run(args: RunArgs, mut output: impl WriteColor, tests: Vec<Test>) -> Result<bool> {
    let jobs = match args.jobs {
        Some(0) => 1,
        Some(jobs) => jobs,
        None => thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1),
    };

    let runtime = Runtime::new()?;
    let mut queue: VecDeque<(String, _)> = VecDeque::with_capacity(jobs);
    let mut at_least_one = false;
    let mut all_passed = true;

    for test in tests.into_iter().filter(|test| args.filter.matches(test)) {
        at_least_one = true;

        if queue.len() >= jobs {
            let (name, join) = queue.pop_front().unwrap();
            all_passed &= report(&mut output, &name, join, &runtime)?;
        }

        let args = args.broker.clone();
        let name = test.name.clone();
        let join = runtime.spawn(run_test(args, test));

        queue.push_back((name, join));
    }

    for (name, join) in queue {
        all_passed &= report(&mut output, &name, join, &runtime)?;
    }

    if at_least_one {
        Ok(all_passed)
    } else {
        println!("No test was selected by the supplied filters (-n,--name and -m,--message).");
        Ok(false)
    }
}

async fn run_test(args: BrokerRunArgs, test: Test) -> Result<Duration, RunError> {
    let mut broker = Broker::new(
        args.broker.as_os_str(),
        Duration::from_millis(args.startup_timeout),
    )
    .await
    .map_err(RunError::without_stderr)?;

    let start = Instant::now();
    let timeout = start + Duration::from_millis(args.timeout);
    let test_res = test.run(&broker, timeout).await;
    let dur = start.elapsed();

    let shutdown_timeout = Instant::now() + Duration::from_millis(args.shutdown_timeout);
    let shutdown_res = broker.shut_down(shutdown_timeout).await;

    match test_res.and(shutdown_res) {
        Ok(()) => Ok(dur),

        Err(e) => {
            if let Ok(stderr) = broker.take_stderr(shutdown_timeout).await {
                Err(RunError::with_stderr(e, stderr))
            } else {
                Err(RunError::without_stderr(e))
            }
        }
    }
}

fn report(
    mut output: impl WriteColor,
    name: &str,
    join: JoinHandle<Result<Duration, RunError>>,
    runtime: &Runtime,
) -> Result<bool> {
    output::prepare_report(&mut output, name)?;
    let res = runtime.block_on(join)?;
    let passed = res.is_ok();
    output::finish_report(output, res)?;
    Ok(passed)
}
