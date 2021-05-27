mod broker;
mod client;
mod output;
mod test;

use anyhow::Result;
use clap::{AppSettings, Clap};
use output::ColorChoice;
use std::process;

#[derive(Clap)]
#[clap(version, author, about,
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::VersionlessSubcommands,
)]
struct Args {
    /// When to color output
    #[clap(long, default_value = "auto", possible_values = &["auto", "always", "never"])]
    color: ColorChoice,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    /// Broker testing
    Broker(broker::Args),

    /// Client testing
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
