mod check;
mod diag;
mod rust;

use anyhow::Result;
use clap::{AppSettings, Clap};
use std::convert::Infallible;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

#[derive(Clap)]
#[clap(version, author, about,
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::DisableVersionForSubcommands,
)]
enum Args {
    /// Checks an Aldrin schema for errors
    Check(check::CheckArgs),

    /// Generates code for Rust
    Rust(rust::RustArgs),
}

#[derive(Clap)]
pub struct CommonArgs {
    /// When to color output
    #[clap(long, default_value = "auto")]
    #[clap(possible_values = &["auto", "always", "never"])]
    color: Color,
}

#[derive(Copy, Clone)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl FromStr for Color {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Infallible> {
        match s {
            "auto" => Ok(Color::Auto),
            "always" => Ok(Color::Always),
            "never" => Ok(Color::Never),
            _ => unreachable!(),
        }
    }
}

#[derive(Clap)]
pub struct CommonReadArgs {
    /// Additional include directories
    #[clap(short = 'I', long, name = "include_dir")]
    #[clap(number_of_values(1))]
    include: Vec<PathBuf>,
}

#[derive(Clap)]
pub struct CommonGenArgs {
    /// Output directory
    ///
    /// Files in the output directory will not be overwritten unless --overwrite is specified.
    #[clap(short, long = "output", name = "output_dir")]
    output_dir: PathBuf,

    /// Overwrite output files
    #[clap(short = 'f', long)]
    overwrite: bool,

    /// Skip generating client-side code for services
    #[clap(long)]
    no_client: bool,

    /// Skip generating server-side code for services
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
