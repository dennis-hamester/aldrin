mod broker;
mod client;
mod output;
mod test;

use anyhow::Result;
use clap::Parser;
use output::ColorChoice;
use std::process;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// When to color output.
    #[clap(long, default_value_t = ColorChoice::Auto)]
    #[arg(value_enum)]
    color: ColorChoice,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Broker testing.
    #[clap(subcommand)]
    Broker(broker::Args),

    /// Client testing.
    #[clap(subcommand)]
    Client(client::Args),
}

fn main() -> Result<()> {
    let res = {
        let args = Args::parse();
        let output = output::make_output(args.color)?;

        match args.command {
            Command::Broker(args) => broker::run(output, args)?,
            Command::Client(args) => client::run(output, args)?,
        }
    };

    if !res {
        process::exit(1);
    }

    Ok(())
}
