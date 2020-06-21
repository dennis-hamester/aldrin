mod check;
mod rust;

use std::path::PathBuf;
use std::process;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
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

#[derive(StructOpt, Debug)]
pub struct CommonReadArgs {
    /// Additional include directories
    #[structopt(short = "I", long, name = "include_dir")]
    #[structopt(number_of_values(1))]
    include: Vec<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct CommonGenArgs {
    /// Output directory
    ///
    /// Files in the output directory will not be overwritten unless --overwrite is specified.
    #[structopt(short, long = "output", name = "output_dir")]
    output_dir: PathBuf,

    /// Overwrite output file
    #[structopt(short = "f", long)]
    overwrite: bool,

    /// Skip generating client-side code for services
    #[structopt(long = "no-client", parse(from_flag = std::ops::Not::not))]
    client: bool,

    /// Skip generating server-side code for services
    #[structopt(long = "no-server", parse(from_flag = std::ops::Not::not))]
    server: bool,
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
