mod check;
mod diag;
mod rust;

use std::convert::Infallible;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    about,
    global_settings = &[AppSettings::VersionlessSubcommands, AppSettings::ColoredHelp]
)]
enum Args {
    /// Checks an Aldrin schema for errors
    Check(check::CheckArgs),

    /// Generates code for Rust
    Rust(rust::RustArgs),
}

#[derive(StructOpt)]
pub struct CommonArgs {
    /// When to color output
    #[structopt(long, default_value = "auto")]
    #[structopt(possible_values = &["auto", "always", "never"])]
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

#[derive(StructOpt)]
pub struct CommonReadArgs {
    /// Additional include directories
    #[structopt(short = "I", long, name = "include_dir")]
    #[structopt(number_of_values(1))]
    include: Vec<PathBuf>,
}

#[derive(StructOpt)]
pub struct CommonGenArgs {
    /// Output directory
    ///
    /// Files in the output directory will not be overwritten unless --overwrite is specified.
    #[structopt(short, long = "output", name = "output_dir")]
    output_dir: PathBuf,

    /// Overwrite output files
    #[structopt(short = "f", long)]
    overwrite: bool,

    /// Skip generating client-side code for services
    #[structopt(long)]
    no_client: bool,

    /// Skip generating server-side code for services
    #[structopt(long)]
    no_server: bool,
}

fn main() {
    let res = match Args::from_args() {
        Args::Check(args) => check::run(args),
        Args::Rust(args) => rust::run(args),
    };

    let exit_code = match res {
        Ok(()) => 0,
        Err(()) => 1,
    };

    process::exit(exit_code);
}
