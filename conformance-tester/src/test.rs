use crate::output;
use anyhow::{anyhow, Error, Result};
use clap::Parser;
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use termcolor::WriteColor;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

#[derive(Parser)]
pub struct CommonRunArgs {
    /// Maximum number of tests to run in parallel
    ///
    /// If unspecified, then the number of CPUs is used.
    #[clap(short, long)]
    jobs: Option<usize>,

    /// Filter tests by name
    ///
    /// Tests will be filtered by sub-string matches. If multiple filters are supplied, a test will
    /// be selected if it matches at least one filter (logical or).
    #[clap(short, long, number_of_values = 1)]
    test: Vec<String>,

    /// Filter tests by message
    ///
    /// If multiple messages are supplied, a test will be selected if it involves at least one of
    /// the supplied messages.
    ///
    /// Valid messages are: connect, connect-reply and shutdown.
    #[clap(short, long, number_of_values = 1)]
    message: Vec<MessageType>,
}

pub struct RunError {
    pub error: Error,
    pub stderr: Vec<u8>,
}

impl RunError {
    pub fn bare(err: impl Into<Error>) -> Self {
        RunError {
            error: err.into(),
            stderr: Vec::new(),
        }
    }

    pub fn with_stderr(error: impl Into<Error>, stderr: Vec<u8>) -> Self {
        RunError {
            error: error.into(),
            stderr,
        }
    }

    pub fn context<C>(self, context: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        RunError {
            error: self.error.context(context),
            stderr: self.stderr,
        }
    }
}

pub type RunBox<Args> =
    Box<dyn FnOnce(Args) -> Pin<Box<dyn Future<Output = Result<(), RunError>> + Send>>>;

pub trait Test {
    type Args;

    fn name(&self) -> &'static str;
    fn short(&self) -> &'static str;
    fn long(&self) -> Option<&'static str>;
    fn message_types(&self) -> &[MessageType];
    fn run(&mut self) -> Option<RunBox<Self::Args>>;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageType {
    Connect,
    ConnectReply,
    Shutdown,
    CreateChannel,
    CreateChannelReply,
    DestroyChannelEnd,
    DestroyChannelEndReply,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Connect => f.pad("connect"),
            Self::ConnectReply => f.pad("connect-reply"),
            Self::Shutdown => f.pad("shutdown"),
            Self::CreateChannel => f.pad("create-channel"),
            Self::CreateChannelReply => f.pad("create-channel-reply"),
            Self::DestroyChannelEnd => f.pad("destroy-channel-end"),
            Self::DestroyChannelEndReply => f.pad("destroy-channel-end-reply"),
        }
    }
}

impl FromStr for MessageType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "connect" => Ok(Self::Connect),
            "connect-reply" => Ok(Self::ConnectReply),
            "shutdown" => Ok(Self::Shutdown),
            "create-channel" => Ok(Self::CreateChannel),
            "create-channel-reply" => Ok(Self::CreateChannelReply),
            "destroy-channel-end" => Ok(Self::DestroyChannelEnd),
            "destroy-channel-end-reply" => Ok(Self::DestroyChannelEndReply),
            _ => Err(anyhow!("invalid message")),
        }
    }
}

pub fn run<W, A, I>(mut output: W, args: CommonRunArgs, test_args: A, tests: I) -> Result<bool>
where
    W: WriteColor,
    I: IntoIterator,
    I::Item: Test<Args = A>,
    A: Clone,
{
    let jobs = match args.jobs {
        Some(0) => 1,
        Some(jobs) => jobs,
        None => num_cpus::get(),
    };

    let runtime = Runtime::new()?;
    let mut queue = VecDeque::with_capacity(jobs);
    let mut at_least_one = false;
    let mut all_passed = true;

    for mut test in tests {
        if !args.test.is_empty() {
            let name = test.name();
            if !args.test.iter().any(|t| name.contains(t)) {
                continue;
            }
        }

        if !args.message.is_empty() {
            let message_types = test.message_types();
            if !args.message.iter().any(|m| message_types.contains(m)) {
                continue;
            }
        }

        at_least_one = true;

        if queue.len() >= jobs {
            let (name, join) = queue.pop_front().unwrap();
            all_passed &= report(&mut output, name, join, &runtime)?;
        }

        let run_box = test.run().unwrap();
        let fut = run_box(test_args.clone());
        let join = runtime.spawn(fut);
        queue.push_back((test.name(), join));
    }

    for (name, join) in queue {
        all_passed &= report(&mut output, name, join, &runtime)?;
    }

    if !at_least_one {
        println!("No test was selected by the supplied filters (-t,--test and -m,--message).");
    }

    Ok(all_passed)
}

fn report(
    mut output: impl WriteColor,
    name: &str,
    join: JoinHandle<Result<(), RunError>>,
    runtime: &Runtime,
) -> Result<bool> {
    output::prepare_report(&mut output, name)?;
    let res = runtime.block_on(join)?;
    let passed = res.is_ok();
    output::finish_report(output, res)?;
    Ok(passed)
}
