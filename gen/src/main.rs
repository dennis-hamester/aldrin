mod check;
mod diag;
mod fmt;
mod rust;

use anyhow::Result;
use clap::Parser;
use colorchoice_clap::Color;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    #[clap(flatten)]
    color: Color,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Checks an Aldrin schema for errors.
    Check(check::CheckArgs),

    /// Formats an Aldrin schema.
    Fmt(fmt::FmtArgs),

    /// Generates code for Rust.
    Rust(rust::RustArgs),
}

#[derive(Parser)]
struct CommonReadArgs {
    /// Additional include directories.
    ///
    /// Can be specified multiple times.
    #[clap(short = 'I', long)]
    include: Vec<PathBuf>,
}

#[derive(Parser)]
struct CommonGenArgs {
    /// Output directory.
    ///
    /// The current working directory will be used if this is not specified.
    #[clap(short, long = "output")]
    output_dir: Option<PathBuf>,

    /// Skip generating client-side code for services.
    #[clap(long)]
    no_client: bool,

    /// Skip generating server-side code for services.
    #[clap(long)]
    no_server: bool,

    /// Generate introspection for all services and types.
    #[clap(long)]
    introspection: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    args.color.write_global();

    let res = match args.cmd {
        Command::Check(args) => check::run(args)?,
        Command::Fmt(args) => fmt::run(args)?,
        Command::Rust(args) => rust::run(args)?,
    };

    if !res {
        process::exit(1);
    }

    Ok(())
}
