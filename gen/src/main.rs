mod check;
mod diag;
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
    /// The current working directory will be used if this is not specified. Files in the output
    /// directory will not be overwritten unless --overwrite is specified.
    #[clap(short, long = "output")]
    output_dir: Option<PathBuf>,

    /// Overwrite output files.
    #[clap(short = 'f', long)]
    overwrite: bool,

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
        Command::Rust(args) => rust::run(args)?,
    };

    if !res {
        process::exit(1);
    }

    Ok(())
}
