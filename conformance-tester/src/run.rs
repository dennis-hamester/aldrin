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

    let mut total = 0;
    let mut passed = 0;

    for test in tests.into_iter().filter(|test| args.filter.matches(test)) {
        total += 1;

        if queue.len() >= jobs {
            let (name, join) = queue.pop_front().unwrap();

            if report(&mut output, &name, join, &runtime)? {
                passed += 1;
            }
        }

        let args = args.broker.clone();
        let name = test.name.clone();
        let join = runtime.spawn(run_test(args, test));

        queue.push_back((name, join));
    }

    for (name, join) in queue {
        if report(&mut output, &name, join, &runtime)? {
            passed += 1;
        }
    }

    if total > 0 {
        output::summary(&mut output, passed, total)?;
        Ok(passed == total)
    } else {
        println!(
            "No test was selected by the supplied filters \
                  (-n,--name, -m,--message and -p,--version)."
        );
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

    let shutdown_timeout = Instant::now() + Duration::from_millis(args.shutdown_timeout);
    let shutdown_res = broker.shut_down(shutdown_timeout).await;
    let dur = start.elapsed();

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
