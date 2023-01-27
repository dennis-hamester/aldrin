mod check;
mod diag;
mod rust;

use anyhow::Result;
use clap::{ColorChoice, Parser};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[clap(version, about)]
enum Args {
    /// Checks an Aldrin schema for errors.
    Check(check::CheckArgs),

    /// Generates code for Rust.
    Rust(rust::RustArgs),
}

#[derive(Parser)]
pub struct CommonArgs {
    /// When to color output.
    #[clap(long, default_value_t = ColorChoice::Auto)]
    #[arg(value_enum)]
    color: ColorChoice,
}

#[derive(Parser)]
pub struct CommonReadArgs {
    /// Additional include directories.
    ///
    /// Can be specified multiple times.
    #[clap(short = 'I', long)]
    include: Vec<PathBuf>,
}

#[derive(Parser)]
pub struct CommonGenArgs {
    /// Output directory.
    ///
    /// Files in the output directory will not be overwritten unless --overwrite is specified.
    #[clap(short, long = "output")]
    output_dir: PathBuf,

    /// Overwrite output files.
    #[clap(short = 'f', long)]
    overwrite: bool,

    /// Skip generating client-side code for services.
    #[clap(long)]
    no_client: bool,

    /// Skip generating server-side code for services.
    #[clap(long)]
    no_server: bool,
}

fn main() -> Result<()> {
    let res = match Args::parse() {
        Args::Check(args) => check::run(args)?,
        Args::Rust(args) => rust::run(args)?,
    };

    if !res {
        process::exit(1);
    }

    Ok(())
}
