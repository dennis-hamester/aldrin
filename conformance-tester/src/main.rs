mod broker;
mod client;
mod client_id;
mod context;
mod message;
mod message_type;
mod output;
mod run;
mod run_error;
mod serial;
mod test;
mod util;
mod uuid_ref;
mod value;

use anyhow::{anyhow, Result};
use clap::{ColorChoice, Parser};
use message_type::MessageType;
use std::path::PathBuf;
use std::process::ExitCode;
use test::Test;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// When to color output.
    #[clap(long, default_value_t = ColorChoice::Auto)]
    #[arg(value_enum)]
    color: ColorChoice,

    /// Path to tests to use instead of the built-in ones.
    ///
    /// This can point to a single JSON test file or a directory of such files.
    #[clap(short, long)]
    tests: Option<PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub struct FilterArgs {
    /// Select only tests with one of the specified names.
    #[clap(short, long)]
    name: Vec<String>,

    /// Select only tests with one of the specified messages.
    #[clap(short, long)]
    message: Vec<MessageType>,
}

impl FilterArgs {
    fn matches(&self, test: &Test) -> bool {
        (!self.name.is_empty() && self.name.iter().any(|name| test.name == *name))
            || (!self.message.is_empty()
                && self
                    .message
                    .iter()
                    .any(|msg| test.message_types.contains(msg)))
            || (self.name.is_empty() && self.message.is_empty())
    }
}

#[derive(Parser)]
pub struct DescribeArgs {
    /// Name of the test to describe.
    test: String,
}

#[derive(Clone, Parser)]
pub struct BrokerRunArgs {
    /// Timeout in milliseconds until the broker is ready to accept connections.
    ///
    /// The broker must print the port to stdout and then accept connections within this time.
    #[clap(long, default_value_t = 1000)]
    startup_timeout: u64,

    /// Timeout in milliseconds until the broker terminates when closing stdin.
    ///
    /// Stdin will be closed to signal the broker process that it is supposed to shut down.
    #[clap(long, default_value_t = 1000)]
    shutdown_timeout: u64,

    /// Timeout in milliseconds for a test to complete.
    #[clap(long, default_value_t = 1000)]
    timeout: u64,

    /// Path to the broker
    broker: PathBuf,
}

#[derive(Parser)]
pub struct RunArgs {
    #[clap(flatten)]
    filter: FilterArgs,

    #[clap(flatten)]
    broker: BrokerRunArgs,

    /// Maximum number of tests to run in parallel.
    ///
    /// If unspecified, then the number of CPUs is used.
    #[clap(short, long)]
    jobs: Option<usize>,
}

#[derive(Parser)]
enum Command {
    /// List tests with a short description and relevant messages.
    List(FilterArgs),

    /// Describe a test in detail.
    Describe(DescribeArgs),

    /// Run conformance tests.
    Run(RunArgs),
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();
    let output = output::make_output(args.color)?;
    let tests = test::get_tests(args.tests.as_deref())?;

    match args.command {
        Command::List(args) => {
            output::list_tests(args, output, &tests)?;
            Ok(ExitCode::SUCCESS)
        }

        Command::Describe(args) => {
            let test = tests
                .iter()
                .find(|test| test.name == args.test)
                .ok_or_else(|| anyhow!("test `{}` not found", args.test))?;
            output::describe_test(output, test)?;
            Ok(ExitCode::SUCCESS)
        }

        Command::Run(args) => {
            if run::run(args, output, tests.into_owned())? {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::FAILURE)
            }
        }
    }
}
