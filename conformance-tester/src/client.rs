mod client_under_test;
mod tests;

use crate::test::{CommonRunArgs, Test};
use crate::{output, test};
use anyhow::{anyhow, Result};
use clap::Parser;
use client_under_test::ClientUnderTest;
use termcolor::WriteColor;

#[derive(Parser)]
pub enum Args {
    /// Lists available client tests.
    List,

    /// Describes a test in more detail.
    Describe(DescribeArgs),

    /// Runs client tests.
    Run(RunArgs),
}

#[derive(Parser)]
#[clap(arg_required_else_help = true)]
pub struct RunArgs {
    #[clap(flatten)]
    common: CommonRunArgs,

    #[clap(flatten)]
    run_args: ClientRunArgs,
}

#[derive(Clone, Parser)]
pub struct ClientRunArgs {
    /// Path to the client.
    client: String,

    /// Timeout in milliseconds for a test.
    #[clap(long, default_value_t = 1000)]
    timeout: u64,

    /// Timeout in milliseconds for the client to shut down.
    ///
    /// When a test fails, then the client will be asked to shut down. If it fails to shut down
    /// within the specified amount of time, then the process will be killed.
    #[clap(long, default_value_t = 1000)]
    shutdown_timeout: u64,
}

#[derive(Parser)]
#[clap(arg_required_else_help = true)]
pub struct DescribeArgs {
    /// Name of the test to describe.
    test: String,
}

pub fn run(output: impl WriteColor, args: Args) -> Result<bool> {
    match args {
        Args::List => {
            list(output)?;
            Ok(true)
        }

        Args::Describe(args) => {
            describe(output, &args.test)?;
            Ok(true)
        }

        Args::Run(args) => run_tests(output, args),
    }
}

fn list(output: impl WriteColor) -> Result<()> {
    let tests = tests::make_tests();
    output::list_tests(output, tests)?;
    Ok(())
}

fn describe(output: impl WriteColor, test: &str) -> Result<()> {
    let test = tests::make_tests()
        .into_iter()
        .find(|t| t.name() == test)
        .ok_or_else(|| anyhow!("unknown client test `{}`", test))?;
    output::describe_test(output, test)?;
    Ok(())
}

fn run_tests(output: impl WriteColor, args: RunArgs) -> Result<bool> {
    let tests = tests::make_tests();
    let all_passed = test::run(output, args.common, args.run_args, tests)?;
    Ok(all_passed)
}
